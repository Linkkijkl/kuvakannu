use actix_web::{HttpResponse, error, get, web};
use image::{EncodableLayout, ImageReader, Limits};
use std::path::{Path, PathBuf};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(thumbnail);
}

pub const SUPPORTED_FILE_TYPES: [&str; 15] = [
    "jpg", "jpeg", "png", "webp", "bmp", "dds", "exr", "ff", "gif", "hdr", "ico", "pnm", "qoi",
    "tga", "tiff",
];

#[get("/thumb/{path:.*}")]
pub async fn thumbnail(path: web::Path<String>) -> Result<HttpResponse, actix_web::Error> {
    let path = &path.to_string();
    let path = Path::new(path);
    let internal_path = Path::new("content").join(path);
    let metadata = if let Ok(a) = async_fs::metadata(&internal_path).await {
        a
    } else {
        return Ok(HttpResponse::NotFound().body("File not found"));
    };

    if metadata.is_file() {
        let thumbnail_bytes = get_thumbnail(&internal_path)?;

        // Return thumbnail from memory
        return Ok(HttpResponse::Ok()
            .insert_header(("Content-type", "image/webp"))
            .body(thumbnail_bytes));
    }

    Err(error::ErrorNotFound("File not found"))
}

fn get_thumbnail(file_path: &PathBuf) -> Result<Vec<u8>, actix_web::Error> {
    const MAX_IMAGE_RESOLUTION: u32 = 10_000;
    const THUMBNAIL_SIZE: u32 = 500;
    const THUMBNAIL_QUALITY: f32 = 80.0;

    // Check if file type is supported before attempting decode
    let extension = file_path
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_else(|| panic!("File name {:?} is not valid uniocode", file_path))
        .split(".")
        .last()
        .unwrap_or_default()
        .to_lowercase();
    if !SUPPORTED_FILE_TYPES.contains(&extension.as_str()) {
        return Err(error::ErrorUnprocessableEntity("File type not supported"));
    }

    let mut decoder = ImageReader::open(file_path)
        .map_err(error::ErrorInternalServerError)?
        .with_guessed_format()
        .map_err(error::ErrorInternalServerError)?;

    // Prevent loading too large images before they are decoded into memory
    let mut limits = Limits::default();
    limits.max_alloc = Some(512 * 1024 * 1024); /* 512 MiB */
    limits.max_image_height = Some(MAX_IMAGE_RESOLUTION);
    limits.max_image_width = Some(MAX_IMAGE_RESOLUTION);
    decoder.limits(limits);

    // Generate thumbnail for image
    let img = decoder.decode().map_err(|_| {
        error::ErrorInternalServerError(format!(
            "Could not decode {}. File might be too large or corrupted.",
            file_path
                .file_name()
                .and_then(|a| a.to_str())
                .unwrap_or("!! FILE NAME NOT VALID UNICODE !!")
        ))
    })?;
    let thumb = img.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
    let thumbnail_bytes = webp::Encoder::from_image(&thumb)
        .expect("Something went wrong while encoding webp")
        .encode(THUMBNAIL_QUALITY)
        .as_bytes()
        .to_vec();

    Ok(thumbnail_bytes)
}
