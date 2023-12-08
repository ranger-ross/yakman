use crate::model::{LabelType, YakManRole};
use crate::{
    api::is_alphanumeric_kebab_case, error::CreateLabelError, error::YakManApiError,
    middleware::roles::YakManRoleBinding, StateManager,
};
use actix_web::{get, put, web, HttpResponse, Responder};
use actix_web_grants::permissions::AuthDetails;
use log::error;

/// List of all labels
#[utoipa::path(responses((status = 200, body = Vec<LabelType>)))]
#[get("/v1/labels")]
pub async fn get_labels(
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let service = state.get_service();
    let data = service.get_labels().await?;
    return Ok(web::Json(data));
}

/// Create a new label
#[utoipa::path(request_body = LabelType, responses((status = 200, body = String)))]
#[put("/v1/labels")]
pub async fn create_label(
    auth_details: AuthDetails<YakManRoleBinding>,
    label_type: web::Json<LabelType>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    let service = state.get_service();
    let mut label_type = label_type.into_inner();
    label_type.name = label_type.name.to_lowercase();

    if !YakManRoleBinding::has_any_global_role(
        vec![YakManRole::Admin, YakManRole::Approver],
        &auth_details.permissions,
    ) {
        return Err(YakManApiError::forbidden());
    }

    if !is_alphanumeric_kebab_case(&label_type.name) {
        return Err(YakManApiError::bad_request("Invalid label name. Must be alphanumeric kebab case"));
    }

    return match service.create_label(label_type).await {
        Ok(()) => Ok(web::Json(())),
        Err(e) => match e {
            CreateLabelError::DuplicateLabelError { name: _ } => {
                 Err(YakManApiError::bad_request("Duplicate label"))
            }
            CreateLabelError::EmptyOptionsError => {
                // TODO: This does not appear to be working
                 Err(YakManApiError::bad_request("Label must have at least 1 option"))
            }
            CreateLabelError::InvalidPriorityError { prioity } => {
                 Err(YakManApiError::bad_request(&format!("Invalid prioity: {prioity}")))
            }
            CreateLabelError::StorageError { message } => {
                error!("Failed to create label, error: {message}");
                Err(YakManApiError::server_error("Failed to create label"))
            }
        },
    };
}
