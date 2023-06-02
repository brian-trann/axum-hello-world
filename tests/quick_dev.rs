#![allow(unused)]

use anyhow::Result;
use axum::routing::get_service;
use serde_json::json;
#[tokio::test]
async fn quick_dev() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:8080")?;
    hc.do_get("/hello?name=brian").await?.print().await?;
    hc.do_get("/hello_path/brian").await?.print().await?;
    // hc.do_get("/src/main.rs").await?.print().await?;
    let req_login = hc.do_post(
        "/api/login",
        json!({
          "password":"password",
          "username":"demouser",
          "username":"demouser",
        }),
    );
    let req_login_fail = hc.do_post(
        "/api/login",
        json!({
          "username":"foobar",
          "password":"baz"
        }),
    );
    req_login.await?.print().await?;
    // req_login_fail.await?.print().await?;
    let req_create_ticket = hc.do_post(
        "/api/tickets",
        json!({
          "title": "ticket1"
        }),
    );
    req_create_ticket.await?.print().await?;
    // hc.do_delete("/api/tickets/1").await?.print().await?;
    hc.do_get("/api/tickets").await?.print().await?;
    Ok(())
}
