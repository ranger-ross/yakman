use crate::{middleware::roles::YakManRoleBinding, StateManager};
use actix_web::{get, web, HttpResponse};
use actix_web_grants::{permissions::AuthDetails, proc_macro::has_any_role};
use yak_man_core::model::YakManRole;

/// Get config data by instance ID
#[utoipa::path(responses((status = 200, body = String)))]
#[get("/configs/{config_name}/instances/{instance}/data")]
#[has_any_role(
    "YakManRole::Admin",
    "YakManRole::Approver",
    "YakManRole::Operator",
    "YakManRole::Viewer",
    type = "YakManRole"
)]
async fn get_instance_data(
    auth_details: AuthDetails<YakManRoleBinding>,
    path: web::Path<(String, String)>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let (config_name, instance) = path.into_inner();
    let service = state.get_service();

    let config = service.get_config(&config_name).await.unwrap().unwrap(); // TODO: better error handling

    let has_role = YakManRoleBinding::has_any_role(
        vec![
            YakManRole::Admin,
            YakManRole::Approver,
            YakManRole::Operator,
            YakManRole::Viewer,
        ],
        &config.project_uuid,
        auth_details.permissions,
    );

    if !has_role {
        return HttpResponse::Forbidden().finish();
    }

    return match service.get_config_data(&config_name, &instance).await {
        Ok(data) => match data {
            Some((data, content_type)) => HttpResponse::Ok().content_type(content_type).body(data),
            None => HttpResponse::NotFound().body("Instance not found"),
        },
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    };
}
