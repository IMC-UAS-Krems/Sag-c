use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::web::{self, Json};
use actix_web::{get, post, App, HttpServer, Responder, Result};
use rand::Rng;
use sagc::dash::Dash;
use sagc::errors::SagError;
use sagc::grafana::Grafana;
use sagc::parser::parse_input;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Input {
    source: String,
}

#[post("/grafana")]
async fn grafana(input: web::Json<Input>) -> Result<impl Responder, SagError> {
    let result = parse_input(input.source.as_str());

    if let Ok(grafana) = result {
        // dbg!(&grafana);
        let grafana: Grafana = Grafana::from(grafana);
        log::info!("Grafana app compiled successfully!");
        return Ok(Json(grafana));
    }
    log::error!(
        "Grafana app compilation failed with error: {}!",
        result.as_ref().err().unwrap()
    );

    Err(result.err().unwrap())
}

#[post("/dash")]
async fn dash(input: String) -> Result<impl Responder, SagError> {
    let result = parse_input(input.as_str());

    if let Ok(dash) = result {
        let dash: Dash = Dash::from(dash);
        log::info!("Dash app compiled successfully!");
        return Ok(Json(dash));
    }
    log::error!(
        "Dash app compilation failed with error: {}!",
        result.as_ref().err().unwrap()
    );

    Err(result.err().unwrap())
}

#[get("/status")]
async fn status() -> impl Responder {
    const STATUSES: [&str; 5] = [
        "Single",
        "In a relationship",
        "Married",
        "In love",
        "It's complicated",
    ];

    let mut rng = rand::thread_rng();
    let random_status = rng.gen_range(0..5);

    STATUSES[random_status]
}

#[get("/")]
async fn index() -> impl Responder {
    web::Redirect::to("/status").permanent()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allow_any_header()
            .max_age(3600);
        App::new()
            .wrap(cors)
            .service(grafana)
            .service(dash)
            .service(status)
            .service(index)
            .wrap(Logger::default())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
