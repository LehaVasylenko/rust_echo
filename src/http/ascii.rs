use axum::{
    body::Bytes,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum::extract::Query;
use image::GenericImageView;
use crate::model::params::Params;
#[utoipa::path(
    post,
    path = "/rust/ascii",
    summary = "Render an image in ASCII",
    description = r#"Accepts a binary image body (PNG/JPEG/GIF, etc.) and converts it to ASCII art. The scale can be specified using the `scale` query parameter.."#,
    params(
        ("scale" = u32, Query, description = "Scale factor, default 3", example = 3)
    ),
    request_body(
        content = String,
        content_type = "application/octet-stream",
        description = "Raw image bytes; treated as binary."
    ),
    responses(
        (status = 200, description = "Success", body = String),
        (status = 400, description = "Failed to read a picture")
    ),
    tag = "ASCII"
)]
pub async fn ascii_handler(Query(params): Query<Params>, body: Bytes) -> Response {
    let scale = params.get_scale().unwrap_or(3);
    // пробуем прочитать картинку из тела
    let img = match image::load_from_memory(&body) {
        Ok(i) => i,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("cannot read image: {e}"),
            )
                .into_response()
        }
    };

    let ascii = image_to_ascii(&img, scale);
    (
        StatusCode::OK,
        [("content-type", "text/plain; charset=utf-8")],
        ascii,
    )
        .into_response()
}

/// конвертим картинку в ASCII
fn image_to_ascii(img: &image::DynamicImage, scale: u32) -> String {
    let (w, h) = img.dimensions();
    let mut out = String::new();

    for y in 0..h {
        if y % scale != 0 {
            continue;
        }
        for x in 0..w {
            if x % scale != 0 {
                continue;
            }
            let p = img.get_pixel(x, y);
            // усреднение яркости
            let bright = ((p[0] as u16 + p[1] as u16 + p[2] as u16) / 3) as u8;

            // если прозрачный — рисуем пробел
            if p[3] == 0 {
                out.push(' ');
            } else {
                out.push(bright_to_char(bright));
            }
        }
        out.push('\n');
    }

    out
}

fn bright_to_char(b: u8) -> char {
    // от светлого к тёмному
    const MAP: &[u8] = b" .:-=+*#@";
    let idx = (b as usize * (MAP.len() - 1)) / 255;
    MAP[idx] as char
}