use axum::extract::Multipart;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::path::PathBuf;
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::{error, info, warn};

#[utoipa::path(
    post,
    path = "/rust/upload",
    summary = "Upload file",
    description = r#"Accepts big files up to 3 Gb"#,
    request_body(
        content_type = "multipart/form-data"
    ),
    responses(
        (status = 200, description = "Uploaded"),
        (status = 400, description = "Failed to read file"),
        (status = 500, description = "Drain the water")
    ),
    tag = "Upload"
)]
pub async fn upload(mut multipart: Multipart) -> Response {
    let mut saw_any_part = false;
    let mut files_log = Vec::<FileLog>::new();
    let mut form_log = Vec::<FormLog>::new();

    // как и раньше — создаём папку uploads (один раз на запрос)
    let upload_dir = PathBuf::from("uploads");
    if let Err(e) = tokio::fs::create_dir_all(&upload_dir).await {
        error!("failed to create uploads dir: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    while let Ok(Some(mut field)) = multipart.next_field().await {
        saw_any_part = true;

        let field_name = field
            .name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "<unnamed>".to_string());

        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        if let Some(file_name) = field.file_name().map(|s| s.to_string()) {
            // === ФАЙЛОВЫЙ PART: сохраняем на диск, как раньше ===
            let mut file_path = upload_dir.clone();
            file_path.push(&file_name);

            let mut file = match File::create(&file_path).await {
                Ok(f) => f,
                Err(e) => {
                    error!("failed to create file {file_path:?}: {e}");
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            };

            let mut size: u64 = 0;

            while let Ok(Some(chunk)) = field.chunk().await {
                size += chunk.len() as u64;
                if let Err(e) = file.write_all(&chunk).await {
                    error!("failed to write chunk to {file_path:?}: {e}");
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }

            info!("saved file to {:?}", file_path);

            files_log.push(FileLog {
                field_name,
                file_name,
                content_type,
                size,
                path: file_path,
            });
        } else {
            // === Обычное поле формы (как httpbin "form") ===
            match field.text().await {
                Ok(value) => {
                    form_log.push(FormLog {
                        field_name,
                        content_type,
                        size: value.len(),
                    });
                }
                Err(e) => {
                    error!("failed to read text field {field_name}: {e}");
                    return StatusCode::BAD_REQUEST.into_response();
                }
            }
        }
    }

    // Если parts вообще не было (пустой multipart / пустое тело)
    if !saw_any_part {
        warn!("multipart upload: empty request (no parts)");
        return (StatusCode::BAD_REQUEST, "multipart body is empty").into_response();
    }

    // Сводка "как httpbin": что было в files, что в form
    info!("multipart upload summary: files={files_log:#?}, form={form_log:#?}");

    (StatusCode::OK, format!("{:#?}\n{:#?}", files_log, form_log)).into_response()
}

#[derive(Debug)]
struct FileLog {
    field_name: String,
    file_name: String,
    content_type: String,
    size: u64,
    path: PathBuf,
}

#[derive(Debug)]
struct FormLog {
    field_name: String,
    content_type: String,
    size: usize,
}
