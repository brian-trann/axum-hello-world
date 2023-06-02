use crate::{Error, Result, web};
use axum::{routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};

pub fn routes() -> Router {
    Router::new().route("/api/login", post(login_api))
}

async fn login_api(cookies: Cookies, payload: Json<LoginPayload>) -> Result<Json<Value>> {
    println!("->> {:<12} - api_login", "HANDLER");
    // insert real auth
    if payload.username != "demouser" || payload.password != "password" {
        return Err(Error::LoginFail);
    }
    // send cookies
    cookies.add(Cookie::new(web::AUTH_TOKEN, "user-1.exp.sign"));
    // send real body
    let body = Json(json!({
      "result": {"success":true}
    }));

    Ok(body)
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}
