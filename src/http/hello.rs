use std::thread;
use axum::http::Version;
use axum::response::IntoResponse;
use chrono::Local;
use log::info;

#[utoipa::path(
    get,
    path = "/rust/hello",
    summary = "Hello",
    description = r#"Returns 'Hello World!'. Be happy "#,
    request_body = String,
    responses(
        (status = 200, description = "Hello World", body = String),
        (status = 400, description = "Failed"),
        (status = 500, description = "Drain the water")
    ),
    tag = "Echo"
)]
#[inline(always)]
pub async fn hello() -> &'static str {
    "Hello, world!"
}

#[utoipa::path(
    get, head, put, delete, patch, options, post,
    path = "/rust/test",
    summary = "Check parallelism",
    description = r#"Accepts any request body with any method"#,
    request_body = String,
    responses(
        (status = 200, description = "Returns some info with thread and http version", body = String),
        (status = 400, description = "Failed"),
        (status = 500, description = "Drain the water")
    ),
    tag = "Echo"
)]
#[inline(always)]
pub async fn test_handler(v: Version) -> impl IntoResponse {
    let start_time = Local::now().format("%H:%M:%S.3f").to_string();
    info!("[{}] REQUEST: {:?} {:?}", start_time, thread::current().id(), v);
    "OK"
}