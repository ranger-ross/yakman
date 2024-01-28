use crate::{error::YakManApiError, middleware::YakManPrinciple, StateManager};
use actix_web::{
    post,
    web::{self, Json},
    Responder,
};
pub use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatePasswordResetLink {
    pub user_uuid: String,
}

/// Create a new password reset link
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/auth/create-reset-password-link")]
pub async fn create_password_reset_link(
    payload: Json<PasswordResetPayload>,
    state: web::Data<StateManager>,
    principle: YakManPrinciple,
) -> Result<impl Responder, YakManApiError> {
    let target_user_uuid = &payload.password_reset_id;

    let user_uuid = match principle.user_uuid {
        Some(user_id) => user_id,
        None => return Err(YakManApiError::unauthorized()),
    };

    if &user_uuid != target_user_uuid {
        todo!("need to verify current user is global admin");
    }

    match state
        .get_service()
        .create_password_reset_link(&user_uuid)
        .await
    {
        Ok(reset_link) => return Ok(web::Json(reset_link)),
        Err(err) => {
            log::error!("failed to create password reset link: {}", err);
            return Err(YakManApiError::server_error("internal server error"));
        }
    };
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PasswordResetPayload {
    pub password_reset_id: String,
}

/// Setup new user after set password link
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/auth/reset-password")]
pub async fn auth_reset_password(
    payload: Json<PasswordResetPayload>,
    state: web::Data<StateManager>,
) -> Result<impl Responder, YakManApiError> {
    todo!();

    Ok(web::Json(()))
}
