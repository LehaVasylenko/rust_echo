use axum::{
    extract::Multipart,
    http::StatusCode,
    response::IntoResponse,
};
use futures_util::stream::StreamExt;
use tokio::{fs::File, io::AsyncWriteExt};
use std::path::PathBuf;
use tracing::{error, info};

pub async fn upload(mut multipart: Multipart) -> impl IntoResponse {
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "upload.bin".to_string());

        let mut file_path = PathBuf::from("uploads");
        if let Err(e) = tokio::fs::create_dir_all(&file_path).await {
            error!("failed to create uploads dir: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }

        file_path.push(&file_name);

        let mut file = match File::create(&file_path).await {
            Ok(f) => f,
            Err(e) => {
                error!("failed to create file: {e}");
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        };

        let mut field = field;
        while let Ok(Some(chunk)) = field.chunk().await {
            if let Err(e) = file.write_all(&chunk).await {
                error!("failed to write chunk: {e}");
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }

        info!("saved file to {:?}", file_path);
    }

    StatusCode::OK
}
