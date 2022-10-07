use actix_web::http::StatusCode;
use actix_web::{middleware, web, App, HttpResponse, HttpResponseBuilder, HttpServer};

mod calculate;

use crate::calculate::{calc, Edges};

async fn index(item: web::Json<Edges>) -> HttpResponse {
    let item: Vec<_> = item
        .0
         .0
        .iter()
        .map(|(s, e)| (s.as_str(), e.as_str()))
        .collect();
    let result = calc(item);
    match result {
        Ok(x) => HttpResponse::Ok().json(x),
        Err(e) => HttpResponseBuilder::new(StatusCode::BAD_REQUEST).body(e.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .app_data(web::JsonConfig::default().limit(4096)) // <- limit size of the payload (global configuration)
            .service(web::resource("/calculate").route(web::post().to(index)))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use actix_web::{body::to_bytes, dev::Service, http, test, web, App};

    use super::*;

    #[actix_web::test]
    async fn test_index() {
        let app =
            test::init_service(App::new().service(web::resource("/").route(web::post().to(index))))
                .await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&Edges(vec![
                ("IND".to_string(), "EWR".to_string()),
                ("SFO".to_string(), "ATL".to_string()),
                ("GSO".to_string(), "IND".to_string()),
                ("ATL".to_string(), "GSO".to_string()),
            ]))
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(body_bytes, r##"["SFO","EWR"]"##);
    }
}
