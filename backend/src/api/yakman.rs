use crate::{error::YakManApiError, settings};
use actix_web::{get, web, Responder};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct YakManSettingsResponse {
    pub enable_oauth: bool,
}

/// Get YakMan application configurations
#[utoipa::path(responses((status = 200, body = YakManSettingsResponse)))]
#[get("/yakman/settings")]
pub async fn yakman_settings() -> Result<impl Responder, YakManApiError> {
    let enable_oauth = settings::is_oauth_enabled();

    return Ok(web::Json(YakManSettingsResponse { enable_oauth }));
}
