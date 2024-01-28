use crate::{error::YakManApiError, StateManager};
use actix_web::{
    post,
    web::{self, Json},
    Responder,
};
pub use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

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
