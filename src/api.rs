use std::{path::Path};
use futures_lite::stream::StreamExt;

use actix_web::{Error, HttpResponse, get, web};
use serde::Serialize;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(hello_world)
            .service(get_directory),
    );
}

#[get("/hello")]
pub async fn hello_world() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello world!"))
}

#[derive(Debug, Serialize)]
struct DirectoryListing {
    info: Option<String>,
    files: Vec<String>,
    directories: Vec<String>,
}

#[get("/dir/{path:.*}")]
pub async fn get_directory(path: web::Path<String>) -> Result<HttpResponse, Error> {
    let directory = Path::new("content").join(path.to_string());
    let mut directories = vec![];
    let mut files = vec![];
    let mut info = None;
    let mut entries = async_fs::read_dir(directory).await?;
    while let Some(entry) = entries.try_next().await? {
        let a = entry.file_type().await?;
        if a.is_dir()
            && let Ok(a) = entry.file_name().into_string() {
                directories.push(a);
            }
        if a.is_file() {
            if entry.file_name() == "readme.md" {
                let content_bytes = async_fs::read(entry.path()).await?;
                let content = String::from_utf8(content_bytes);
                if let Ok(content) = content {
                    info = Some(markdown::to_html(&content));
                }
                continue;
            }
            else if let Ok(a) = entry.file_name().into_string() {
                files.push(format!("/content/{a}"));
            }
        }
    }

    Ok(HttpResponse::Ok().json(DirectoryListing {
        info,
        files,
        directories,
    }))
}

#[cfg(test)]
mod tests {
    use reqwest::Result;
    const URL: &str = "http://backend:3030";

    // Test if api is available
    #[test]
    fn api_is_responsive() -> Result<()> {
        let status = reqwest::blocking::get(format!("{URL}/api/v1/hello"))?.status();
        assert_eq!(
            status, 200,
            "Could not reach API. Make sure the backend is running and available in `{URL}` before running tests."
        );
        Ok(())
    }
}
