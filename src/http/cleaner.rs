use std::path::PathBuf;
use axum::http::StatusCode;
use axum::response::IntoResponse;

#[utoipa::path(
    get,
    path = "/rust/clean",
    summary = "Clean uploads",
    description = r#"Clean up uploaded files"#,
    responses(
        (status = 200, description = "Cleaned"),
        (status = 204, description = "Already cleaned"),
        (status = 500, description = "Drain the water")
    ),
    tag = "Upload"
)]
pub async fn cleaner() -> impl IntoResponse {
    let file_path = PathBuf::from("uploads");
    match std::fs::remove_dir_all(&file_path) {
        Ok(_) => StatusCode::OK,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => StatusCode::NO_CONTENT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}