use std::str::FromStr;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::web::{self, Json};
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder, Result};
use awc::http::Uri;
use awc::Client;
use rand::seq::IteratorRandom;
use rand::Rng;
use sagc::dash::Dash;
use sagc::errors::{CompileError, GeneralError, SagError, WebErrorPosition};
use sagc::grafana::Grafana;
use sagc::parser::{parse_input, Position};
use sagc::sections::DashboardType;
use serde::{Deserialize, Serialize};

type GrafanaUri = Uri;
type DeployUri = Uri;

#[derive(Debug, Deserialize, Serialize)]
struct Input {
    source: String,
    user_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct DeployPayload {
    source: String,
    user_id: String,
    dashboard_type: String,
}

#[derive(Debug, Serialize)]
struct NoErrors {
    status: String,
}

#[derive(Debug, Serialize)]
enum DashboardResponse {
    Grafana(Grafana),
    Dash(Dash),
}

#[derive(Debug, Serialize)]
struct UrlResponse {
    url: String,
    status: String,
}

impl Responder for DashboardResponse {
    type Body = actix_web::body::BoxBody;
    fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse {
        match self {
            DashboardResponse::Grafana(grafana_json) => HttpResponse::Ok().json(grafana_json),
            DashboardResponse::Dash(dash_json) => HttpResponse::Ok().json(dash_json),
        }
    }
}

async fn fetch_grafana_model(
    client: &Client,
    uri: &Uri,
    grafana_json: &Grafana,
) -> Result<serde_json::Value, GeneralError> {
    let response = client.get(uri).send_json(grafana_json).await;
    let mut response = response.unwrap();
    if response.status().is_success() {
        let body = response.json::<serde_json::Value>().await.unwrap();
        Ok(body)
    } else {
        log::error!(
            "Error ({}): {}",
            response.status(),
            String::from_utf8(response.body().await.unwrap().to_ascii_lowercase()).unwrap()
        );
        Err(GeneralError::new(
            "Error fetching Grafana model".to_string(),
        ))
    }
}

async fn deploy(
    client: &Client,
    uri: &Uri,
    payload: DeployPayload,
) -> Result<String, GeneralError> {
    let response = client.post(uri).send_json(&payload).await;
    let mut response = response.unwrap();
    if response.status().is_success() {
        Ok(String::from_utf8(response.body().await.unwrap().to_vec()).unwrap())
    } else {
        log::error!(
            "Error ({}): {}",
            response.status(),
            String::from_utf8(response.body().await.unwrap().to_ascii_lowercase()).unwrap()
        );
        Err(GeneralError::new("Error deploying".to_string()))
    }
}

#[post("/check")]
async fn check(input: web::Json<Input>) -> Result<impl Responder, WebErrorPosition> {
    let result = parse_input(input.source.as_str());

    if let Err(errors) = result {
        let error = WebErrorPosition {
            status: "error".to_string(),
            errors,
        };
        return Err(error);
    }

    Ok(Json(NoErrors {
        status: "ok".to_string(),
    }))
}

#[post("/compile")]
async fn compile(
    input: web::Json<Input>,
    client: web::Data<Client>,
    grafana_url: web::Data<GrafanaUri>,
    deploy_url: web::Data<DeployUri>,
) -> Result<impl Responder, CompileError> {
    let result = parse_input(input.source.as_str());

    if let Ok(config) = result {
        // dbg!(&grafana);
        let dashboard_type = match config.application.dashboard {
            DashboardType::Grafana => "grafana",
            DashboardType::Dash => "dash",
        };
        let deploy_layload: serde_json::Value = match config.application.dashboard {
            DashboardType::Grafana => {
                let grafana_json: Grafana = Grafana::from(config);
                log::info!("Grafana app compiled successfully!");
                fetch_grafana_model(&client, &grafana_url, &grafana_json)
                    .await
                    .map_err(CompileError::General)?
            }
            DashboardType::Dash => {
                let dash_json: Dash = Dash::from(config);
                log::info!("Dash app compiled successfully!");
                serde_json::json!(dash_json)
            }
        };
        let deploy_layload = DeployPayload {
            source: deploy_layload.to_string(),
            user_id: input.user_id.clone(),
            dashboard_type: dashboard_type.to_string(),
        };
        let response = deploy(&client, &deploy_url, deploy_layload)
            .await
            .map_err(CompileError::General)?;

        Ok(Json(UrlResponse {
            url: response,
            status: "ok".to_string(),
        }))
    } else {
        let error = WebErrorPosition {
            status: "error".to_string(),
            errors: result.err().unwrap(),
        };

        Err(CompileError::WebPos(error))
    }
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

    let grafana_url = std::env::var("GRAFANA_URL").unwrap_or("http://localhost:9000".to_string());
    let deploy_url = std::env::var("DEPLOY_URL").unwrap_or("http://localhost:9001".to_string());
    let grafan_url: GrafanaUri = GrafanaUri::from_str(grafana_url.as_str()).unwrap();
    let deploy_url: DeployUri = DeployUri::from_str(deploy_url.as_str()).unwrap();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allow_any_header()
            .max_age(3600);
        App::new()
            .app_data(web::Data::new(Client::default()))
            .app_data(web::Data::new(grafan_url.clone()))
            .app_data(web::Data::new(deploy_url.clone()))
            .wrap(cors)
            .service(grafana)
            .service(dash)
            .service(status)
            .service(check)
            .service(index)
            .service(compile)
            .service(test)
            .wrap(Logger::default())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
