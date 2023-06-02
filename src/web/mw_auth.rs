use crate::ctx::Ctx;
use crate::model::ModelController;
use crate::{ctx, web::AUTH_TOKEN, Error, Result};
use async_trait::async_trait;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum::RequestPartsExt;
use lazy_regex::regex_captures;
use tower_cookies::{Cookie, Cookies};

pub async fn mw_require_auth<B>(
    // cookies: Cookies,
    ctx: Result<Ctx>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response> {
    // this section moved to the ctx extractor
    println!("->> {:<12} - mw_require_auth - {ctx:?}", "MIDDLEWARE");
    // let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());
    // auth_token
    //     .ok_or(Error::AuthFailNoTokenCookie)
    //     .and_then(parse_token)?;

    // token componetn validation
    ctx?;
    Ok(next.run(req).await)
}
// parse a token of format `user-[user-id].[expiration].[signature]
// return (user_id, expiration, signature)

fn parse_token(token: String) -> Result<(u64, String, String)> {
    let (_whole, user_id, exp, sign) = regex_captures!(r#"^user-(\d+)\.(.+)\.(.+)"#, &token)
        .ok_or(Error::AuthFailTokenWrongFormat)?;
    let user_id: u64 = user_id
        .parse()
        .map_err(|_| Error::AuthFailTokenWrongFormat)?;
    Ok((user_id, exp.to_string(), sign.to_string()))
}
pub async fn mw_ctx_resolver<B>(
    _mc: State<ModelController>,
    cookies: Cookies,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response> {
    println!("->> {:<12} - mw_ctx_resolver", "MIDDLEWARE");
    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());
    let result_ctx = match auth_token
        .ok_or(Error::AuthFailNoTokenCookie)
        .and_then(parse_token)
    {
        Ok((user_id, _exp, _sign)) => {
            // actually do the token validation here; because expensive
            // we may need to do async
            Ok(Ctx::new(user_id))
        }
        Err(e) => Err(e),
    };
    // remove the cookie if something went wrong other than noauthtokencookie
    if result_ctx.is_err() && !matches!(result_ctx, Err(Error::AuthFailNoTokenCookie)) {
        cookies.remove(Cookie::named(AUTH_TOKEN))
    }

    // store ctx_result
    req.extensions_mut().insert(result_ctx); // has to be unique by type
    Ok(next.run(req).await)
}
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        println!("->> {:<12} - Ctx", "EXTRACTOR");
        // THIS IS NOW MOVED TO ctx resolver
        // // user the cookies extractor
        // let cookies = parts.extract::<Cookies>().await.unwrap();
        // let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());
        // let (user_id, exp, sign) = auth_token
        //     .ok_or(Error::AuthFailNoTokenCookie)
        //     .and_then(parse_token)?;
        // // token component validation
        // Ok(Ctx::new(user_id))

        parts
            .extensions
            .get::<Result<Ctx>>()
            .ok_or(Error::AuthFailedCtxNotInRequestExt)?
            .clone()
    }
}
