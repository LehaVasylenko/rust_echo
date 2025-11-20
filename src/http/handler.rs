use axum::{
    body::{to_bytes, Bytes},
    extract::{Query, State},
    http::{HeaderMap, Request, StatusCode},
    response::IntoResponse,
    Json,
};
use base64::{Engine};
use std::collections::HashMap;
use crate::state::AppState;
use crate::model::body_kind::BodyKind;
use crate::model::echo_response::EchoResponse;

pub async fn health() -> &'static str {
    "ok"
}

#[utoipa::path(
    get, post, put, delete, patch, options, head,
    path = "/rust/echo",
    summary = "Echo request",
    description = r#"Accepts any request body with any method, including a file. If a file is returned, its contents will be returned in Base 64 format."#,
    request_body = String,
    responses(
        (status = 200, description = "Echoed request summary", body = EchoResponse),
        (status = 400, description = "Failed to read body")
    ),
    tag = "Echo"
)]
pub async fn echo(
    State(_): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    req: Request<axum::body::Body>,
) -> impl IntoResponse {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let content_length = req
        .headers()
        .get(axum::http::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    // нормализуем заголовки в HashMap
    let headers_map: HashMap<String, String> = headers
        .iter()
        .map(|(k, v)| (k.as_str().to_lowercase(), v.to_str().unwrap_or("<non-utf8>").to_string()))
        .collect();

    // читаем тело
    let (parts, body) = req.into_parts();
    let bytes: Bytes = match to_bytes(body, 80 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => return (StatusCode::BAD_REQUEST, "failed to read body").into_response(),
    };

    let content_type = parts
        .headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let (body_kind, body_json, body_text, body_base64) = if bytes.is_empty() {
        (BodyKind::Empty, None, None, None)
    } else if content_type.starts_with("application/json") {
        match serde_json::from_slice::<serde_json::Value>(&bytes) {
            Ok(v) => (BodyKind::Json, Some(v), None, None),
            Err(_) => fallback_text_or_base64(bytes),
        }
    } else {
        fallback_text_or_base64(bytes)
    };

    let resp = EchoResponse::new(
        method,
        path,
        query,
        headers_map,
        body_kind,
        body_json,
        body_text,
        body_base64,
        content_length,
    );

    (StatusCode::OK, Json(resp)).into_response()
}

fn fallback_text_or_base64(bytes: Bytes) -> (BodyKind, Option<serde_json::Value>, Option<String>, Option<String>) {
    match std::str::from_utf8(&bytes) {
        Ok(s) => (BodyKind::TextUtf8, None, Some(s.to_string()), None),
        Err(_) => (
            BodyKind::Base64,
            None,
            None,
            Some(base64::engine::general_purpose::STANDARD.encode(&bytes)),
        ),
    }
}
