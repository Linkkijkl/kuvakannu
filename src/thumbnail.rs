use actix_web::{HttpResponse, get, web, error};
use image::{EncodableLayout, ImageReader, Limits};
use std::path::Path;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(thumbnail);
}

#[get("/thumb/{path:.*}")]
pub async fn thumbnail(path: web::Path<String>) -> Result<HttpResponse, actix_web::Error> {
    const MAX_IMAGE_RESOLUTION: u32 = 10_000;
    const THUMBNAIL_SIZE: u32 = 500;
    const THUMBNAIL_QUALITY: f32 = 80.0;
    
    let path = &path.to_string();
    let path = Path::new(path);
    let internal_path = Path::new("content").join(path);
    println!("{:?}", internal_path);
    let metadata = if let Ok(a) = async_fs::metadata(&internal_path).await {
        a
    } else {
        return Ok(HttpResponse::NotFound().body("File not found"));
    };

    if metadata.is_file() {
        // Prevent loading "zip bomb" images before the image is decoded in memory
        let mut decoder = ImageReader::open(&internal_path)
            .map_err(error::ErrorInternalServerError)?
            .with_guessed_format()
            .map_err(error::ErrorInternalServerError)?;
        let mut limits = Limits::default();
        limits.max_alloc = Some(512 * 1024 * 1024); /* 512 MiB */
        limits.max_image_height = Some(MAX_IMAGE_RESOLUTION);
        limits.max_image_width = Some(MAX_IMAGE_RESOLUTION);
        decoder.limits(limits);

        // Generate thumbnail for image
        let img = decoder.decode().map_err(|_| {
            error::ErrorBadRequest(format!(
                "Could not decode {}. File might be too large or corrupted.",
                path.file_name().and_then(|a| a.to_str()).unwrap_or("!! FILE NAME NOT VALID UNICODE !!")
            ))
        })?;
        let thumbnail = img.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
        let thumbnail_bytes = webp::Encoder::from_image(&thumbnail)
            .unwrap()
            .encode(THUMBNAIL_QUALITY)
            .as_bytes()
            .to_vec();

        // Return thumbnail from memory
        return Ok(HttpResponse::Ok().insert_header(("Content-type", "image/webp")).body(thumbnail_bytes))
    }
    
    Ok(HttpResponse::Ok().body("eyy"))
}
