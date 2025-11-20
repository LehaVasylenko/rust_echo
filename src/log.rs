use std::time::Instant;
use axum::body::Body;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use tracing::info;

pub async fn log_request(req: Request<Body>, next: Next<>) -> Response {
    let path = req.uri().path().to_string();
    let method = req.method().to_string();
    let start = Instant:: now();
    let res = next.run(req).await;
    let status = res.status().as_u16();
    let elapsed = start.elapsed().as_nanos();
    info!(%method, %path, %elapsed, %status, "request finished");
    res
}