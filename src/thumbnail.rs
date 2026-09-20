use actix_web::{HttpResponse, error, get, web};
use image::{EncodableLayout, ImageReader, Limits};
use log::info;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

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
    if let Ok(metadata) = tokio::fs::metadata(&internal_path).await
        && !metadata.is_file()
    {
        return Err(error::ErrorNotFound("File not found"));
    };

    // Try to read pre-generated thumbnail
    let thumbnail_path = Path::new("thumbnail").join(path);
    let thumbnail_file = match File::open(&thumbnail_path).await {
        Ok(file) => {
            // File exists, return file
            file
        }
        Err(_) => {
            // Thumbnail does not exist, generate it
            let thumbnail_bytes = generate_thumbnail(&internal_path);
            let thumbnail_bytes = match thumbnail_bytes {
                Ok(a) => a,
                Err(_) => return Ok(HttpResponse::TemporaryRedirect().insert_header(("Location", "/static/material/broken-image.svg")).finish()),
            };

            let file_write_result = async {
                let parent_dir = thumbnail_path
                    .parent()
                    .unwrap_or_else(|| panic!("File path has no parent dir: {:?}", thumbnail_path));
                println!("{:?}", parent_dir);
                if !tokio::fs::try_exists(parent_dir).await? {
                    tokio::fs::create_dir_all(parent_dir).await?;
                }
                let mut file = File::create(&thumbnail_path).await?;
                file.write_all(&thumbnail_bytes).await?;
                Ok::<(), std::io::Error>(())
            }
            .await;
            if let Err(e) = file_write_result {
                // Thumbnail writing failed, return bytes from memory
                info!("Error while writing thumbnail: {:?}", e);
                return Ok(HttpResponse::Ok()
                    .insert_header(("Content-type", "image/webp"))
                    .body(thumbnail_bytes));
            }

            File::open(&thumbnail_path)
                .await
                .expect("Could not load thumbnail from disk that was just saved")
        }
    };

    // Return thumbnail file stream
    Ok(HttpResponse::Ok()
        .insert_header(("Content-type", "image/webp"))
        .streaming(tokio_util::io::ReaderStream::new(thumbnail_file)))
}

fn generate_thumbnail(file_path: &PathBuf) -> Result<Vec<u8>, actix_web::Error> {
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
