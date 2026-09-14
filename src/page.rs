use actix_web::{error, HttpResponse, get, web};
use async_recursion::async_recursion;
use sailfish::TemplateSimple;
use std::path::{Path, PathBuf};
use tokio::fs::DirEntry;

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

struct Breadcrumb {
    pub href: String,
    pub name: String,
}

#[derive(TemplateSimple)]
#[template(path = "listing.stpl")]
struct ListingTemplate {
    info: String,
    files: Vec<File>,
    directories: Vec<Directory>,
    breadcrumbs: Vec<Breadcrumb>,
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
            if let Ok(file_type) = entry.file_type().await {
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
    let metadata = tokio::fs::metadata(&internal_file_path).await?;

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

    // Return error if the request path does not match any file or directory in filesystem
    if !metadata.is_dir() {
        return Err(error::ErrorNotFound("File not found"));
    }

    // Get and sort directory listing
    let mut file_entries: Vec<DirEntry> = vec![];
    let mut dir_entries: Vec<DirEntry> = vec![];
    let mut fs_entry_stream = tokio::fs::read_dir(internal_file_path).await?;
    while let Some(entry) = fs_entry_stream.next_entry().await? {
        let entry_type = entry.file_type().await?;
        if entry_type.is_file() {
            file_entries.push(entry);
        }
        else if entry_type.is_dir() {
            dir_entries.push(entry);
        }
    }
    file_entries.sort_unstable_by_key(|a| a.file_name().to_ascii_lowercase());
    dir_entries.sort_unstable_by_key(|a| a.file_name().to_ascii_lowercase());

    // Construct Directory objects from listing directories
    let mut directories = vec![];
    for entry in dir_entries {
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
            None => "/static/material/folder.svg".to_string(),
        };
        directories.push(Directory {
            name: dir_name,
            public_path: dir_public_path.to_string(),
            thumbnail_path,
        });
    }

    // Construct File types and info from listing files
    let mut files = vec![];
    let mut info = None;
    for entry in file_entries {
        // Render readme markdown to listing info html
        if entry.file_name() == "readme.md" {
            let content = tokio::fs::read_to_string(entry.path()).await?;
            info = Some(markdown::to_html(&content));
            continue;
        }

        // Cosntruct File
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

    // Construct breadcrumbs for request path
    let mut breadcrumbs = Vec::with_capacity(request_path.len());
    let empty = Breadcrumb {
        name: String::new(),
        href: String::new(),
    };
    for entry in request_path.iter() {
        let next_path = format!("{}/{}", breadcrumbs.last().unwrap_or(&empty).href, entry);
        breadcrumbs.push(Breadcrumb {
            href: next_path,
            name: entry.to_string(),
        });
    }

    // Render page
    let rendered_page = ListingTemplate {
        info: info.unwrap_or_default(),
        files,
        directories,
        breadcrumbs,
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
