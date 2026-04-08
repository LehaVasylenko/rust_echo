use std::sync::Arc;
use std::thread;
use axum::extract::State;
use axum::http::Version;
use axum::response::IntoResponse;
use chrono::Local;
use log::info;
use crate::state::AppState;

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
    tag = "Test"
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
    tag = "Test"
)]
#[inline(always)]
pub async fn test_handler(State(state): State<Arc<AppState>>, v: Version) -> impl IntoResponse {
    let start_time = Local::now().format("%H:%M:%S%.3f").to_string();
    let f = format!("[{}] REQUEST: {:?} {:?}", start_time, thread::current().id(), v);
    let current_sleep = state.sleeptime();
    tokio::time::sleep(std::time::Duration::from_millis(current_sleep)).await;
    info!("{}", &f);
    f
}

#[utoipa::path(
    get,
    path = "/rust/sleep/{ms:u64}",
    summary = "Check parallelism",
    description = r#"Accepts ms to sleep for /rust/test"#,
    responses(
        (status = 200, description = "OK", body = String),
        (status = 400, description = "Failed"),
        (status = 500, description = "Drain the water")
    ),
    tag = "Test"
)]
#[inline(always)]
pub async fn set_sleep(State(state): State<Arc<AppState>>, axum::extract::Path(ms): axum::extract::Path<u64>) -> impl IntoResponse {
    state.set_sleeptime(ms);
    ms.to_string()
}