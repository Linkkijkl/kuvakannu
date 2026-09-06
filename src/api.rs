use actix_web::{Error, HttpResponse, get, web};

#[get("/hello")]
pub async fn hello_world() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Hello world!"))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(hello_world),
    );
}

#[cfg(test)]
mod tests {
    use reqwest::Result;
    const URL: &str = "http://backend:3030";

    // Test if api is available
    #[test]
    fn api_is_responsive() -> Result<()> {
        let status = reqwest::blocking::get(format!("{URL}/api/v1/hello"))?.status();
        assert_eq!(status, 200, "Could not reach API. Make sure the backend is running and available in `{URL}` before running tests.");
        Ok(())
    }
}
