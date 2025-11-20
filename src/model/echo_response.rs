use std::collections::HashMap;
use serde::Serialize;
use utoipa::{ToResponse, ToSchema};
use crate::model::body_kind::BodyKind;

#[derive(Serialize, ToSchema, ToResponse, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EchoResponse {
    method: String,
    path: String,
    query: HashMap<String, String>,
    headers: HashMap<String, String>,
    body_kind: BodyKind,
    body_json: Option<serde_json::Value>,
    body_text: Option<String>,
    body_base64: Option<String>,
    content_length: Option<u64>,
}

impl EchoResponse {
    pub fn new(method: String,
               path: String,
               query: HashMap<String, String>,
               headers: HashMap<String, String>,
               body_kind: BodyKind,
               body_json: Option<serde_json::Value>,
               body_text: Option<String>,
               body_base64: Option<String>,
               content_length: Option<u64>,
    ) -> Self {
        EchoResponse {
            method,
            path,
            query,
            headers,
            body_kind,
            body_json,
            body_text,
            body_base64,
            content_length,
        }
    }
}