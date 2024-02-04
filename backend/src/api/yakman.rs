use crate::error::YakManApiError;
use actix_web::{get, web, Responder};
pub use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct YakManConfigResponse {
    pub enable_oauth: bool,
}

/// Get YakMan application configurations
#[utoipa::path(responses((status = 200, body = String)))]
#[get("/yakman/config")]
pub async fn yakman_config() -> Result<impl Responder, YakManApiError> {
    let enable_oauth = std::env::var("YAKMAN_OAUTH_ENABLED")
        .map(|v| v.parse::<bool>().ok())
        .ok()
        .flatten()
        .unwrap_or_default();

    return Ok(web::Json(YakManConfigResponse { enable_oauth }));
}
