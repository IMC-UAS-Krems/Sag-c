use actix_web::middleware::Logger;
use actix_web::web::Json;
use actix_web::{post, App, HttpServer, Responder, Result};
use sagc::dash::Dash;
use sagc::errors::SagError;
use sagc::grafana::Grafana;
use sagc::parser::parse_input;

#[post("/grafana")]
async fn grafana(input: String) -> Result<impl Responder, SagError> {
    let result = parse_input(input.as_str());
    match result {
        Ok(grafana) => {
            // dbg!(&grafana);
            let grafana: Grafana = Grafana::from(grafana);
            Ok(Json(grafana))
        }
        Err(e) => Err(e),
    }
}

#[post("/dash")]
async fn dash(input: String) -> Result<impl Responder, SagError> {
    let result = parse_input(input.as_str());
    match result {
        Ok(dash) => {
            let dash: Dash = Dash::from(dash);
            Ok(Json(dash))
        }
        Err(e) => Err(e),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(grafana)
            .service(dash)
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
