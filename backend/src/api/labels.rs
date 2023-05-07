use crate::{
    api::is_alphanumeric_kebab_case, middleware::roles::YakManRoleBinding,
    services::errors::CreateLabelError, StateManager, YakManError,
};
use actix_web::{get, put, web, HttpResponse, Responder};
use actix_web_grants::permissions::AuthDetails;
use log::error;
use yak_man_core::model::{LabelType, YakManRole};

/// List of all labels
#[utoipa::path(responses((status = 200, body = Vec<LabelType>)))]
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

/// Create a new label
#[utoipa::path(request_body = LabelType, responses((status = 200, body = String)))]
#[put("/labels")]
pub async fn create_label(
    auth_details: AuthDetails<YakManRoleBinding>,
    label_type: web::Json<LabelType>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let service = state.get_service();
    let mut label_type = label_type.into_inner();
    label_type.name = label_type.name.to_lowercase();

    if !YakManRoleBinding::has_any_global_role(
        vec![YakManRole::Admin, YakManRole::Approver],
        &auth_details.permissions,
    ) {
        return HttpResponse::Forbidden().finish();
    }

    if !is_alphanumeric_kebab_case(&label_type.name) {
        return HttpResponse::BadRequest()
            .body("Invalid label name. Must be alphanumeric kebab case");
    }

    return match service.create_label(label_type).await {
        Ok(()) => HttpResponse::Ok().finish(),
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
                error!("Failed to create label, error: {message}");
                HttpResponse::InternalServerError().body("Failed to create label")
            }
        },
    };
}
