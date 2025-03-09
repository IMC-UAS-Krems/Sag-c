use std::str::FromStr;
use std::time::Duration;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::web::{self, Json};
use actix_web::{get, post, App, HttpServer, Responder, Result, error::ErrorInternalServerError};
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

// ------ new imports from include snippets ----------
use dotenv::dotenv;
use std::env::var;

//-----IMPORTS FOR TESTING------
use std::fs::File;
use sagc::preprocessing::{substitute_imports, ImportFileContent, ContentRequest, substitute_imports_from_backend};
use std::io::Error;

#[derive(Debug, Clone)]
struct GrafanaUri(Uri);
#[derive(Debug, Clone)]
struct DeployUri(Uri);

//--------- Edited from Egor's snippets -----------

#[derive(Debug, Deserialize, Serialize)]
struct FileMetadata {
    municipalityName: String,
    #[serde(rename(serialize = "organizationName"))]  // orgName -> organizationName
    orgName: String,
    projectName: String,
    path: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Input {
    source: String,
    user_id: String,
    //metadata: FileMetadata,
}

//-------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
struct DeployPayload {
    source: String,
    user_id: String,
    dashboard_type: String,
    deployments: Vec<String>,
}

#[derive(Debug, Serialize)]
struct NoErrors {
    status: String,
}

#[derive(Debug, Serialize)]
struct UrlResponse {
    url: String,
    status: String,
}

async fn fetch_grafana_model(
    client: &Client,
    uri: &Uri,
    grafana_json: &Grafana,
) -> Result<serde_json::Value, GeneralError> {
    log::info!("Fetching Grafana model from {}...", uri);
    let response = client
        .post(uri)
        .timeout(Duration::new(60 * 5, 0))
        .send_json(grafana_json)
        .await;

    if let Err(e) = response {
        log::error!("Error fetching Grafana model: {}", e);
        return Err(GeneralError::new(
            "Error fetching Grafana model".to_string(),
        ));
    }

    let mut response = response.unwrap();

    if response.status().is_success() {
        let body = response.json::<serde_json::Value>().await.unwrap();
        log::debug!("{}", &body.to_string());
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
    let response = client
        .post(uri)
        .timeout(Duration::new(60 * 5, 0))
        .send_json(&payload)
        .await;

    if let Err(e) = response {
        log::error!("Error deploying dashboard: {}", e);
        return Err(GeneralError::new("Error deploying dashboard".to_string()));
    }

    let mut response = response.unwrap();

    if response.status().is_success() {
        let resp_url = String::from_utf8(response.body().await.unwrap().to_vec())
            .unwrap()
            .replace('"', "");
        log::debug!("{}", &resp_url);
        Ok(resp_url)
    } else {
        log::error!(
            "Error ({}): {}",
            response.status(),
            String::from_utf8(response.body().await.unwrap().to_ascii_lowercase()).unwrap()
        );
        Err(GeneralError::new("Error deploying".to_string()))
    }
}

// ------ Endpoints Collection ------

/// checks json input for errors
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
    deploy_url: web::Data<DeployUri>,
    grafana_url: web::Data<GrafanaUri>,
) -> Result<impl Responder, CompileError> {

    dbg!(&input);

    let result = parse_input(input.source.as_str());

    if let Ok(config) = result {
        // dbg!(&grafana);
        dbg!(&config);
        let dashboard_type = match config.application.dashboard {
            DashboardType::Grafana => "grafana",
            DashboardType::Dash => "dash",
        };
        let deployment_types = config
            .deployment
            .environments
            .values()
            .map(|env| env.r#type.to_string())
            .collect();

        let deploy_layload: serde_json::Value = match config.application.dashboard {
            DashboardType::Grafana => {
                let grafana_json: Grafana = Grafana::from(config);
                log::info!("Grafana app compiled successfully!");
                fetch_grafana_model(&client, &grafana_url.0, &grafana_json)
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
            deployments: deployment_types,
        };

        let response = deploy(&client, &deploy_url.0, deploy_layload)
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

/// compiles a grafana dashboard
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

/// compiles a dash dashboard
#[post("/dash")]
async fn dash(input: web::Json<Input>) -> Result<impl Responder, WebErrorPosition> {
    let result = parse_input(input.source.as_str());

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

/// test error handling
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

/// test connectivity to the backend
#[post("/test/import")]
async fn import_test(input: web::Json<Input>, client: web::Data<Client>) -> Result<String, actix_web::Error> {
    let data = ContentRequest {
        municipalityName: "Krems".into(),
        orgName: "Imc".into(),
        projectName: "Project 1".into(),
        path: "folder-1.file-1".into(),
    };
    dotenv::dotenv().ok(); // Load environment variables

    let token = var("WEB_TOKEN").map_err(|e| ErrorInternalServerError(e))?;
    println!("WEB_TOKEN: {}", token);

    let req_url = "http://localhost:9512/api/document_content";
    let mut req = client.get(req_url).bearer_auth(&token);

    let query_str = serde_json::to_string(&data).unwrap(); // Debugging
    println!("Serialized Query Params (not URL-encoded): {}", query_str);

    req = req.query(&data).map_err(|e| ErrorInternalServerError(e))?;

    println!("Sending request to: {}", req_url);
    println!("BEFORE");
    let mut res = req.send().await.map_err(|e| ErrorInternalServerError(e))?;
    println!("AFTER");
    println!("Response Status: {}", res.status());

    let body_bytes = res.body().await.map_err(|e| ErrorInternalServerError(e))?;
    let body_string = String::from_utf8(body_bytes.to_vec()).map_err(|e| ErrorInternalServerError(e))?;

    println!("Response Body: {}", body_string);

    Ok(body_string)
}

/// import content from backend and subtitute it in the target content
#[post("/test/import_from_backend")]
async fn import_from_backend(client: web::Data<Client>, input: web::Json<Input>) -> Result<String, actix_web::Error> {
    match substitute_imports_from_backend(input.source.as_str()).await {
        Ok(result) => Ok(result),
        Err(errors) => {
            let error_messages: Vec<String> = errors.into_iter().map(|e| e.to_string()).collect();  // TODO: Implement Display for Error
            let error_message = error_messages.join(", ");
            Err(ErrorInternalServerError(error_message))
        },
    }
}

#[post("/test/grafana")]
async fn testgrafana(input: web::Json<Input>) -> Result<impl Responder, WebErrorPosition> {

    let mut files: Vec<ImportFileContent> = Vec::new(); //mock DB ---> this should be provided in advance
    files.push(ImportFileContent { name: "BnB", content: "BnB:
    type is smartcomm-minmaxbarchart-panel
    source is first
    locations -> Escuelas Aguirre, Arturo Soria, Villaverde
    traces -> dateObserved, NOx, O3, NO2" });
    files.push(ImportFileContent { name: "import", content: "#import mock" }); //mock for nested imports to trigger an error

    match substitute_imports(input.source.as_str(), &files) {
        Ok(preprocessed_source) => {
            // Parse the preprocessed input
            let result = parse_input(&preprocessed_source);

            if let Ok(parsed_config) = result {
                let g: Grafana = Grafana::from(parsed_config);
                log::info!("Grafana app compiled successfully!");
                return Ok(Json(g)); // Ensure `Grafana` implements Serialize
            }

            // If parsing fails, return error
            let error = WebErrorPosition {
                status: "error".to_string(),
                errors: result.err().unwrap(),
            };
            Err(error)
        }
        Err(errors) => {
            // Handle preprocessing errors
            let error = WebErrorPosition {
                status: "error".to_string(),
                errors,
            };
            Err(error)
        }
    }
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

    let mut grafana_url =
        std::env::var("GRAFANA_URL").unwrap_or("http://localhost:9005".to_string());
    let mut deploy_url = std::env::var("DEPLOY_URL").unwrap_or("http://localhost:9001".to_string());

    grafana_url.push('/');
    deploy_url.push_str("/deploy");

    let grafan_url: GrafanaUri = GrafanaUri(Uri::from_str(grafana_url.as_str()).unwrap());
    let deploy_url: DeployUri = DeployUri(Uri::from_str(deploy_url.as_str()).unwrap());

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
            .service(import_test)
            .service(testgrafana)
            .service(import_from_backend)
            .wrap(Logger::default())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
