use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::web::{self, Json};
use actix_web::{get, post, App, HttpServer, Responder, Result};
use rand::seq::IteratorRandom;
use rand::Rng;
use sagc::dash::Dash;
use sagc::errors::{SagError, WebErrorPosition};
use sagc::grafana::Grafana;
use sagc::parser::{parse_input, Position};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Input {
    source: String,
}

#[post("/grafana")]
async fn grafana(input: web::Json<Input>) -> Result<impl Responder, WebErrorPosition> {
    let result = parse_input(input.source.as_str());

    if let Ok(grafana) = result {
        // dbg!(&grafana);
        let grafana: Grafana = Grafana::from(grafana);
        log::info!("Grafana app compiled successfully!");
        return Ok(Json(grafana));
    }
    // log::error!(
    //     "Grafana app compilation failed with error: {}!",
    //     result.as_ref().err().unwrap()
    // );
    //
    let error = WebErrorPosition {
        status: "error".to_string(),
        errors: result.err().unwrap(),
    };

    Err(error)
}

#[post("/dash")]
async fn dash(input: String) -> Result<impl Responder, WebErrorPosition> {
    let result = parse_input(input.as_str());

    if let Ok(dash) = result {
        let dash: Dash = Dash::from(dash);
        log::info!("Dash app compiled successfully!");
        return Ok(Json(dash));
    }
    // log::error!(
    //     "Dash app compilation failed with error: {}!",
    //     result.as_ref().err().unwrap()
    // );

    let error = WebErrorPosition {
        status: "error".to_string(),
        errors: result.err().unwrap(),
    };

    Err(error)
}

#[post("/test")]
async fn test(input: web::Json<Input>) -> Result<String, WebErrorPosition> {
    let input = input.source.as_str();
    let lines = input.lines().collect::<Vec<&str>>();
    let mut errors = Vec::new();
    // select 5 random lines
    let mut rng = rand::thread_rng();
    let random_lines =
        (0..lines.len()).choose_multiple(&mut rng, (lines.len().div_euclid(9)).max(2));
    for i_line in random_lines {
        let line = lines[i_line];
        let line_len = line.len();
        // select 2 random positions
        let mut random_positions = (0..line_len).choose_multiple(&mut rng, 2);
        random_positions.sort();
        if random_positions.len() < 2 {
            continue;
        }

        let position = Position {
            row_start: i_line,
            row_end: i_line,
            col_start: *random_positions.get(0).unwrap(),
            col_end: *random_positions.get(1).unwrap(),
        };
        errors.push(SagError::unparsable(position));
    }
    let errors = WebErrorPosition {
        status: "error".to_string(),
        errors,
    };
    Err(errors)
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
            .service(test)
            .wrap(Logger::default())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
