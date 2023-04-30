use crate::StateManager;

use actix_web::{get, post, web, HttpResponse, put};

/// Get all of the revisions for a config
#[utoipa::path(responses((status = 200, body = Vec<ConfigInstanceRevision>)))]
#[get("/configs/{config_name}/instance/{instance}/revisions")]
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

/// Submit a new revision for review
#[utoipa::path(responses((status = 200, body = String)))]
#[put("/configs/{config_name}/instance/{instance}/revision/{revision}/submit")]
async fn submit_instance_revision(
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

/// Approves and applies a revision to a config instance
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/configs/{config_name}/instance/{instance}/revision/{revision}/approve")]
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
