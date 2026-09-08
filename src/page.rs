use actix_web::{HttpResponse, get, web};
use futures_lite::stream::StreamExt;
use sailfish::TemplateSimple;
use std::path::Path;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(page);
}

#[derive(Debug)]
struct File {
    name: String,
    public_path: String,
}

#[derive(Debug)]
struct Directory {
    name: String,
    public_path: String,
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

#[get("/{path:.*}")]
pub async fn page(path: web::Path<String>) -> Result<HttpResponse, actix_web::Error> {
    let path = &path.to_string();
    let path = Path::new(path);
    let public_path = Path::new("content").join(path);
    let internal_path = Path::new("content").join(path);
    let metadata = if let Ok(a) = async_fs::metadata(&internal_path).await {
        a
    } else {
        return Ok(HttpResponse::NotFound().body("File not found"));
    };

    // Generate a item view
    if metadata.is_file() {
        if let Some(file_name) = public_path.file_name()
            && let Some(file_name) = file_name.to_str()
            && let Some(public_path) = public_path.to_str()
        {
            let rendered_page = ItemTemplate {
                file: File {
                    name: String::from(file_name),
                    public_path: String::from(public_path),
                }
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
    let mut entries = async_fs::read_dir(internal_path).await?;
    while let Some(entry) = entries.try_next().await? {
        let a = entry.file_type().await?;
        if a.is_dir()
            && let Ok(a) = entry.file_name().into_string()
            && let Some(dir_public_path) = public_path.join(&a).to_str()
        {
            directories.push(Directory {
                name: a,
                public_path: String::from(dir_public_path),
            });
        }
        if a.is_file() {
            if entry.file_name() == "readme.md" {
                let content = async_fs::read_to_string(entry.path()).await?;
                info = Some(markdown::to_html(&content));
                continue;
            } else if let Ok(a) = entry.file_name().into_string() {
                let file_public_path = public_path.join(&a);
                let file_public_path = file_public_path.to_str().unwrap_or_default();
                let file = File {
                    public_path: String::from(file_public_path),
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
