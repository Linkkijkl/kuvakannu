use actix_web::{HttpResponse, get, web};
use async_recursion::async_recursion;
use sailfish::TemplateSimple;
use std::path::{Path, PathBuf};

use crate::{thumbnail::SUPPORTED_FILE_TYPES, web_path::WebPath};

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
    if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(file_type) = entry.file_type().await
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
                            panic!("File path {:?} is not valid uniocode", entry.path())
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
    let request_path = WebPath::from(path.as_str());
    let mut public_file_path = WebPath::from("content");
    public_file_path.extend(request_path.iter());
    let mut public_thumbnail_path = WebPath::from("thumb");
    public_thumbnail_path.extend(request_path.iter());
    let internal_file_path = Path::new("content").join(request_path.to_string());
    let metadata = if let Ok(a) = tokio::fs::metadata(&internal_file_path).await {
        a
    } else {
        return Ok(HttpResponse::NotFound().body("File not found"));
    };

    // Generate a item view
    if metadata.is_file() {
        let name = request_path
            .last()
            .expect("request path for an existing file does not contain file name")
            .to_string();
        let rendered_page = ItemTemplate {
            file: File {
                name,
                public_content_path: public_file_path.to_string(),
                public_path: request_path.to_string(),
                thumbnail_path: public_thumbnail_path.to_string(),
            },
        }
        .render_once()
        .unwrap();

        return Ok(HttpResponse::Ok().body(rendered_page));
    }

    // Generate a directory listing
    if !metadata.is_dir() {
        return Ok(HttpResponse::NotFound().body("File not found"));
    }

    let mut directories = vec![];
    let mut files = vec![];
    let mut info = None;
    let mut entries = tokio::fs::read_dir(internal_file_path).await?; // TODO: Sort alphabetically
    while let Some(entry) = entries.next_entry().await? {
        let entry_type = entry.file_type().await?;
        if entry_type.is_dir() {
            let dir_name = entry
                .file_name()
                .into_string()
                .unwrap_or_else(|_| panic!("dir path is not valid unicode: {:?}", entry.path()));
            let mut dir_public_path = request_path.clone();
            let dir_name_2 = dir_name.clone();
            dir_public_path.push(&dir_name_2);
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
                name: dir_name,
                public_path: dir_public_path.to_string(),
                thumbnail_path,
            });
        }
        if entry_type.is_file() {
            // Render readme markdown to listing info html
            if entry.file_name() == "readme.md" {
                let content = tokio::fs::read_to_string(entry.path()).await?;
                info = Some(markdown::to_html(&content));
                continue;
            }

            let file_entry_name = entry
                .file_name()
                .into_string()
                .unwrap_or_else(|_| panic!("file name is not valid unicode: {:?}", entry.path()));
            let mut file_public_content_path = public_file_path.clone();
            file_public_content_path.push(&file_entry_name);
            let mut file_thumbnail_path = public_thumbnail_path.clone();
            file_thumbnail_path.push(&file_entry_name);
            let mut file_public_path = request_path.clone();
            file_public_path.push(&file_entry_name);
            let file = File {
                public_content_path: file_public_content_path.to_string(),
                public_path: file_public_path.to_string(),
                thumbnail_path: file_thumbnail_path.to_string(),
                name: file_entry_name,
            };
            files.push(file);
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
