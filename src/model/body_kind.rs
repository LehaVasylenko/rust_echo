use serde::Serialize;
use utoipa::{ToResponse, ToSchema};

#[derive(Serialize, ToSchema, ToResponse, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum BodyKind {
    Json,
    TextUtf8,
    Base64,
    Empty,
}