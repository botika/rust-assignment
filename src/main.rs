use std::fmt::Debug;

use actix_web::http::StatusCode;
use actix_web::web::Json;
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder, ResponseError};

use thiserror::Error;

mod calculate;

use crate::calculate::{AIter, CalcError, WGraph};

#[derive(Debug, Error)]
#[error("{0}")]
struct ResError(#[from] anyhow::Error);

#[rustfmt::skip]
macro_rules! matcher {
    ($_self:ident) => {
        macro_rules! m {
            ($ty:ty => $s:expr) => {
                m!($ty, _ => $s)
            };
            ($ty:ty, $p:pat_param => $s:expr) => {
                if matches!($_self.0.downcast_ref::<$ty>(), Some($p)) {
                    return $s;
                }
            };
        }
    };
}

impl ResponseError for ResError {
    fn status_code(&self) -> StatusCode {
        matcher!(self);
        m!(CalcError, CalcError::Cycle(_) => StatusCode::BAD_REQUEST);
        m!(CalcError => StatusCode::EXPECTATION_FAILED);
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

fn tuple_refs(item: &[(String, String)]) -> impl AIter {
    item.iter().map(|(s, e)| (s.as_str(), e.as_str()))
}

async fn index(item: Json<Vec<(String, String)>>) -> impl Responder {
    Ok::<_, ResError>(HttpResponse::Ok().json(WGraph::calc_first_last(tuple_refs(&item.0))?))
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
    use actix_web::http::StatusCode;
    use actix_web::{body::to_bytes, dev::Service, test, web, App};

    use super::*;

    #[actix_web::test]
    async fn test_index() {
        let app =
            test::init_service(App::new().service(web::resource("/").route(web::post().to(index))))
                .await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json([
                ["IND", "EWR"],
                ["SFO", "ATL"],
                ["GSO", "IND"],
                ["ATL", "GSO"],
            ])
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(body_bytes, r##"["SFO","EWR"]"##);

        let req = test::TestRequest::post()
            .uri("/")
            .set_json([["foo", "foo"]])
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(body_bytes, r##"cycle detected in node "foo""##);
    }
}
