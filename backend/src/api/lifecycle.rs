use crate::error::YakManApiError;
use actix_web::{get, web, Responder};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct YakManHealthResponse {
    pub status: &'static str,
}
impl YakManHealthResponse {
    fn new() -> Self {
        Self { status: "UP" }
    }
}

/// Health check
#[utoipa::path(responses((status = 200, body = YakManHealthResponse)))]
#[get("/health")]
pub async fn health() -> Result<impl Responder, YakManApiError> {
    return Ok(web::Json(YakManHealthResponse::new()));
}
