use std::time::Duration;

use actix_files::Files;
use actix_web::dev::Service;
use actix_web::http::header::{CACHE_CONTROL, HeaderValue};
use actix_web::{App, HttpServer, middleware};

mod page;
mod r#static;
mod thumbnail;
mod web_path;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    colog::init();

    let actix = tokio::task::spawn(
        HttpServer::new(|| {
            let headers_middleware =
                middleware::DefaultHeaders::new().add(("content-type", "text/html; charset=UTF-8"));
            let logger_middleware = middleware::Logger::new("%t %s %r %Dms");

            App::new()
                .wrap(headers_middleware)
                .wrap(logger_middleware)
                .wrap_fn(|req, srv| {
                    let fut = srv.call(req);
                    async {
                        let mut res = fut.await?;
                        res.headers_mut()
                            .insert(CACHE_CONTROL, HeaderValue::from_static("max-age=3600"));
                        Ok(res)
                    }
                })
                .service(Files::new("content", "./content"))
                .configure(thumbnail::config)
                .configure(r#static::config)
                .configure(page::config)
        })
        .keep_alive(Duration::from_secs(60))
        .client_request_timeout(Duration::from_secs(60))
        .bind(("0.0.0.0", 3030))?
        .run(),
    );

    actix.await??;

    Ok(())
}
