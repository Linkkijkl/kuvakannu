use actix_web::{HttpResponse, error, get, mime::TEXT_PLAIN_UTF_8, web};
use include_dir::{Dir, include_dir};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(r#static);
}

static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");

#[get("/static/{path:.*}")]
pub async fn r#static(path: web::Path<String>) -> Result<HttpResponse, actix_web::Error> {
    let contents = STATIC_DIR
        .get_file(path.to_string())
        .ok_or_else(|| error::ErrorNotFound("Static file not found"))?
        .contents();
    let mime = path
        .to_string()
        .split('.')
        .next_back()
        .and_then(|a| mime_guess::from_ext(a).first())
        .unwrap_or(TEXT_PLAIN_UTF_8);
    Ok(HttpResponse::Ok()
        .insert_header(("Content-type", mime.essence_str()))
        .body(contents))
}
