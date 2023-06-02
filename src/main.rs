#![allow(unused)]
use std::net::SocketAddr;

use crate::log::log_request;
use crate::model::ModelController;
use axum::extract::{Path, Query};
use axum::http::{Method, Uri};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, get_service};
use axum::{middleware, Json, Router};
use serde::Deserialize;
use serde_json::json;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use uuid::Uuid;

mod ctx;
mod error;
mod log;
mod model;
mod web;

pub use self::error::{Error, Result};
#[tokio::main]
async fn main() -> Result<()> {
    // init model controller
    let mc = ModelController::new().await?;
    let routes_apis = web::routes_tickets::routes(mc.clone())
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));
    let routes_all: Router = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        .nest("/api", routes_apis)
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            mc.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new()); // layers go bottom top
                                           // .fallback_service(routes_static_fallback()); you probably wont do this tho

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("->> LISTENING on {addr}\n");
    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();
    Ok(())
}
// e.g. as query params :D
async fn handler_hello(params: Query<HelloParams>) -> impl IntoResponse {
    println!("->> {:<12} - handler_hello - {params:?}", "HANDLER");
    let name = params.name.as_deref().unwrap_or("World");
    Html(format!("Hello <strong>{name}</strong>"))
}
// e.g. as path // look at Path(name):Path<String> .. this is destructureing
async fn handler_hello_path(Path(name): Path<String>) -> impl IntoResponse {
    println!("->> {:<12} - handler_hello_path - {name:?}", "HANDLER");

    Html(format!("Hello Path: <strong>{name}</strong>"))
}
// composing multiple routers
fn routes_hello() -> Router {
    Router::new()
        .route("/hello", get(handler_hello))
        .route("/hello_path/:name", get(handler_hello_path))
}

fn routes_static_fallback() -> Router {
    // this gives literally access to the fs cuz the serve dir. idk could be useful if you're specific. maybe to like an openapi.yml
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}
// middlewayer
async fn main_response_mapper(
    ctx: Option<ctx::Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");
    println!();
    let uuid = Uuid::new_v4();
    // get the eventual response error
    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());

    // if client error, build new response
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error" :{
                    "type" :client_error.as_ref(),
                    "req_uuid": uuid.to_string(),
                }
            });
            println!("    ->> client_error_body : {client_error_body}");
            // build new response from the clienterrorbody
            // `*` will deref
            (*status_code, Json(client_error_body)).into_response()
        });
    // todo: build and log the server log line
    let client_error = client_status_error.unzip().1;
    log_request(uuid, req_method, uri, ctx, service_error, client_error).await;
    println!();
    error_response.unwrap_or(res)
}
#[derive(Debug, Deserialize)]
struct HelloParams {
    name: Option<String>,
}
