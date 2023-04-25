use std::collections::HashMap;

use crate::{
    services::errors::{CreateConfigError, CreateConfigInstanceError, CreateLabelError},
    StateManager, YakManError,
};

use actix_web::{get, post, put, web, HttpRequest, HttpResponse, Responder};
use yak_man_core::model::{Label, LabelType};

#[get("/configs")]
pub async fn get_configs(
    state: web::Data<StateManager>,
) -> actix_web::Result<impl Responder, YakManError> {
    let service = state.get_service();

    return match service.get_configs().await {
        Ok(data) => Ok(web::Json(data)),
        Err(err) => Err(YakManError::from(err)),
    };
}

#[get("/labels")]
pub async fn get_labels(
    state: web::Data<StateManager>,
) -> actix_web::Result<impl Responder, YakManError> {
    let service = state.get_service();

    return match service.get_labels().await {
        Ok(data) => Ok(web::Json(data)),
        Err(_) => Err(YakManError::new("Failed to load labels from storage")),
    };
}

#[put("/labels")]
pub async fn create_label(
    label_type: web::Json<LabelType>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let service = state.get_service();

    return match service.create_label(label_type.into_inner()).await {
        Ok(()) => HttpResponse::Ok().body(""),
        Err(e) => match e {
            CreateLabelError::DuplicateLabelError { name: _ } => {
                HttpResponse::BadRequest().body("Duplicate label")
            }
            CreateLabelError::EmptyOptionsError => {
                // TODO: This does not appear to be working
                HttpResponse::BadRequest().body("Label must have at least 1 option")
            }
            CreateLabelError::InvalidPriorityError { prioity } => {
                HttpResponse::BadRequest().body(format!("Invalid prioity: {prioity}"))
            }
            CreateLabelError::StorageError { message } => {
                println!("Failed to create label, error: {message}");
                HttpResponse::InternalServerError().body("Failed to create label")
            }
        },
    };
}

// TODO: Standardize REST endpoint naming

#[get("/config/{config_name}/instance")]
async fn get_data_by_labels(
    path: web::Path<String>,
    query: web::Query<HashMap<String, String>>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let config_name = path.into_inner();
    let service = state.get_service();

    let labels: Vec<Label> = extract_labels(query);

    println!("Search for config {config_name} with labels: {:?}", labels);

    return match service
        .get_config_data_by_labels(&config_name, labels)
        .await
    {
        Ok(data) => {
            if let Some((data, content_type)) = data {
                HttpResponse::Ok().content_type(content_type).body(data)
            } else {
                HttpResponse::NotFound().body("Config not found")
            }
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    };
}

// TODO: Rename method to get Config by ID
#[get("/instances/{config_name}")]
async fn get_instance_by_id(
    path: web::Path<String>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let config_name = path.into_inner();
    let service = state.get_service();
    return match service.get_config_instance_metadata(&config_name).await {
        Ok(data) => match data {
            Some(data) => HttpResponse::Ok().body(serde_json::to_string(&data).unwrap()),
            None => HttpResponse::NotFound().body("Instance not found"),
        },
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    };
}

#[get("/config/{config_name}/instance/{instance}")]
async fn get_instance(
    path: web::Path<(String, String)>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let (config_name, instance) = path.into_inner();
    let service = state.get_service();

    return match service.get_config_data(&config_name, &instance).await {
        Ok(data) => match data {
            Some((data, content_type)) => HttpResponse::Ok().content_type(content_type).body(data),
            None => HttpResponse::NotFound().body("Instance not found"),
        },
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    };
}

#[put("/config/{config_name}/data")] // TODO: Rename to /instance
async fn create_new_instance(
    path: web::Path<String>,
    query: web::Query<HashMap<String, String>>,
    data: String,
    state: web::Data<StateManager>,
    req: HttpRequest,
) -> HttpResponse {
    let config_name = path.into_inner();
    let service = state.get_service();

    let labels: Vec<Label> = extract_labels(query);
    let content_type: Option<String> = get_content_type(&req);

    // TODO: do validation
    // - labels are valid
    // - not a duplicate?

    match service
        .create_config_instance(&config_name, labels, &data, content_type)
        .await
    {
        Ok(_) => HttpResponse::Ok().body(""),
        Err(CreateConfigInstanceError::NoConfigFound) => {
            HttpResponse::BadRequest().body("Invalid config name")
        }
        Err(CreateConfigInstanceError::StorageError { message: _ }) => {
            HttpResponse::InternalServerError().body("Failed to create config")
        }
    }
}

#[put("/config/{config_name}")]
async fn create_config(path: web::Path<String>, state: web::Data<StateManager>) -> HttpResponse {
    let config_name = path.into_inner();
    let service = state.get_service();
    let result = service.create_config(&config_name).await;

    return match result {
        Ok(()) => HttpResponse::Ok().body(""),
        Err(e) => match e {
            CreateConfigError::StorageError { message } => {
                println!("Failed to create config {config_name}, error: {message}");
                HttpResponse::InternalServerError().body("Failed to create config")
            }
            CreateConfigError::DuplicateConfigError { name: _ } => {
                HttpResponse::BadRequest().body("duplicate config")
            }
        },
    };
}

#[post("/config/{config_name}/instance/{instance}")]
async fn update_new_instance(
    path: web::Path<(String, String)>,
    query: web::Query<HashMap<String, String>>,
    data: String,
    state: web::Data<StateManager>,
    req: HttpRequest,
) -> HttpResponse {
    let (config_name, instance) = path.into_inner();
    let service = state.get_service();

    let labels: Vec<Label> = extract_labels(query);
    let content_type: Option<String> = get_content_type(&req);

    // TODO: do validation
    // - config exists
    // - labels are valid
    // - not a duplicate?

    return match service
        .save_config_instance(&config_name, &instance, labels, &data, content_type)
        .await
    {
        Ok(_) => HttpResponse::Ok().body(""),
        Err(_) => HttpResponse::InternalServerError().body("failed to create instance"),
    };
}

#[get("/config/{config_name}/instance/{instance}/revisions")]
async fn get_instance_revisions(
    path: web::Path<(String, String)>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let (config_name, instance) = path.into_inner();
    let service = state.get_service();

    if let Some(data) = service
        .get_instance_revisions(&config_name, &instance)
        .await
        .unwrap()
    {
        return HttpResponse::Ok().body(serde_json::to_string(&data).unwrap());
    }
    return HttpResponse::NotFound().body("");
}

#[post("/config/{config_name}/instance/{instance}/revision/{revision}/current")] // TODO: This should be renamed to /submit
async fn update_instance_current_revision(
    path: web::Path<(String, String, String)>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let (config_name, instance, revision) = path.into_inner();
    let service = state.get_service();

    return match service
        .update_instance_current_revision(&config_name, &instance, &revision)
        .await
    {
        Ok(_) => HttpResponse::Ok().body(""),
        Err(_) => HttpResponse::InternalServerError().body("failed to update instance"),
    };
}

#[post("/config/{config_name}/instance/{instance}/revision/{revision}/approve")]
async fn approve_pending_instance_revision(
    path: web::Path<(String, String, String)>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let (config_name, instance, revision) = path.into_inner();
    let service = state.get_service();

    return match service
        .approve_pending_instance_revision(&config_name, &instance, &revision)
        .await
    {
        Ok(_) => HttpResponse::Ok().body(""),
        Err(_) => HttpResponse::InternalServerError().body("failed to update instance"),
    };
}

fn extract_labels(query: web::Query<HashMap<String, String>>) -> Vec<Label> {
    return query
        .iter()
        .map(|param| Label {
            label_type: param.0.to_string(),
            value: param.1.to_string(),
        })
        .collect();
}

fn get_content_type(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("content-type")?
        .to_str()
        .map(|s| String::from(s))
        .ok()
}
