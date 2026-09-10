use actix_web::{HttpResponse, get, web};
use async_fs::DirEntry;
use async_recursion::async_recursion;
use futures_lite::stream::StreamExt;
use sailfish::TemplateSimple;
use std::path::{Path, PathBuf};

use crate::thumbnail::SUPPORTED_FILE_TYPES;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(page);
}

#[derive(Debug)]
struct File {
    name: String,
    public_path: String,
    public_content_path: String,
    thumbnail_path: String,
}

#[derive(Debug)]
struct Directory {
    name: String,
    public_path: String,
    thumbnail_path: String,
}

#[derive(TemplateSimple)]
#[template(path = "listing.stpl")]
struct ListingTemplate {
    info: String,
    files: Vec<File>,
    directories: Vec<Directory>,
}

#[derive(TemplateSimple)]
#[template(path = "item.stpl")]
struct ItemTemplate {
    file: File,
}

#[async_recursion]
async fn get_first_file_recursive(dir: PathBuf) -> Option<PathBuf> {
    if let Ok(mut entries) = async_fs::read_dir(dir).await {
        while let Some(entry) = entries.next().await {
            if let Ok(entry) = entry
                && let Ok(file_type) = entry.file_type().await
            {
                if file_type.is_dir() {
                    if let Some(a) = get_first_file_recursive(entry.path()).await {
                        return Some(a);
                    }
                } else if file_type.is_file() {
                    let extension = entry
                        .file_name()
                        .to_str()
                        .unwrap_or_else(|| {
                            panic!("File name {:?} is not valid uniocode", entry.path())
                        })
                        .split(".")
                        .last()
                        .unwrap_or_default()
                        .to_lowercase();
                    if SUPPORTED_FILE_TYPES.contains(&extension.as_str()) {
                        return Some(entry.path());
                    }
                }
            }
        }
    }
    None
}

#[get("/{path:.*}")]
pub async fn page(path: web::Path<String>) -> Result<HttpResponse, actix_web::Error> {
    let path = &path.to_string();
    let path = Path::new(path);
    let public_content_path = Path::new("content").join(path);
    let internal_path = Path::new("content").join(path);
    let thumbnail_path = Path::new("thumb").join(path);
    let metadata = if let Ok(a) = async_fs::metadata(&internal_path).await {
        a
    } else {
        return Ok(HttpResponse::NotFound().body("File not found"));
    };

    // Generate a item view
    if metadata.is_file() {
        if let Some(file_name) = public_content_path.file_name()
            && let Some(file_name) = file_name.to_str()
            && let Some(public_path) = public_content_path.to_str()
            && let Some(thumbnail_path) = thumbnail_path.to_str()
            && let Some(path) = path.to_str()
        {
            let rendered_page = ItemTemplate {
                file: File {
                    name: String::from(file_name),
                    public_content_path: String::from(public_path),
                    public_path: String::from(path),
                    thumbnail_path: String::from(thumbnail_path),
                },
            }
            .render_once()
            .unwrap();

            return Ok(HttpResponse::Ok().body(rendered_page));
        }

        return Ok(HttpResponse::NotFound().body("File not found"));
    }

    // Generate a directory listing
    if !metadata.is_dir() {
        return Ok(HttpResponse::NotFound().body("File not found"));
    }

    let mut directories = vec![];
    let mut files = vec![];
    let mut info = None;
    let mut entries = async_fs::read_dir(internal_path).await?; // TODO: Sort alphabetically
    while let Some(entry) = entries.try_next().await? {
        let a = entry.file_type().await?;
        if a.is_dir()
            && let Ok(a) = entry.file_name().into_string()
            && let Some(dir_public_path) = path.join(&a).to_str()
        {
            let thumbnail_path = get_first_file_recursive(entry.path()).await;
            let thumbnail_path = match thumbnail_path {
                Some(thumbnail_path) => PathBuf::from("/thumb")
                    .join(thumbnail_path.iter().skip(1).collect::<PathBuf>())
                    .to_str()
                    .unwrap_or_else(|| panic!("file path is not valid utf8: {:?}", entry.path()))
                    .to_string(),
                None => "/nonexistent".to_string(), // TODO: No thumbnailable image found, use default directory thumbnail
            };
            directories.push(Directory {
                name: a,
                public_path: String::from(dir_public_path),
                thumbnail_path,
            });
        }
        if a.is_file() {
            if entry.file_name() == "readme.md" {
                let content = async_fs::read_to_string(entry.path()).await?;
                info = Some(markdown::to_html(&content));
                continue;
            } else if let Ok(a) = entry.file_name().into_string() {
                let file_public_content_path = public_content_path.join(&a);
                let file_public_content_path =
                    file_public_content_path.to_str().unwrap_or_default();
                let file_thumbnail_path = thumbnail_path.join(&a);
                let file_thumbnail_path = file_thumbnail_path.to_str().unwrap_or_default();
                let file_public_path = entry.path();
                let file_public_path = file_public_path.to_str().unwrap_or_default();
                let file = File {
                    public_content_path: String::from(file_public_content_path),
                    public_path: String::from(file_public_path),
                    thumbnail_path: String::from(file_thumbnail_path),
                    name: a,
                };
                files.push(file);
            }
        }
    }

    let rendered_page = ListingTemplate {
        info: info.unwrap_or_default(),
        files,
        directories,
    }
    .render_once()
    .unwrap();

    Ok(HttpResponse::Ok().body(rendered_page))
}

#[cfg(test)]
mod tests {
    use reqwest::Result;
    const URL: &str = "http://backend:3030";

    // Test if api is available
    #[test]
    fn api_is_responsive() -> Result<()> {
        let status = reqwest::blocking::get(URL)?.status();
        assert_eq!(
            status, 200,
            "Could not reach API. Make sure the backend is running and available in `{URL}` before running tests."
        );
        Ok(())
    }
}
