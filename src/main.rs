use actix_web::{App, HttpServer, middleware};

mod api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let actix = tokio::task::spawn(
        HttpServer::new(|| {
            let headers_middleware =
                middleware::DefaultHeaders::new().add(("content-type", "text/html; charset=UTF-8"));
            let logger_middleware = middleware::Logger::new("%t %s %r %Dms");

            App::new()
                .wrap(headers_middleware)
                .wrap(logger_middleware)
                .configure(api::config)
        })
        .bind(("0.0.0.0", 3030))?
        .run(),
    );

    actix.await??;

    Ok(())
}
