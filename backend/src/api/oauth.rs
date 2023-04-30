use crate::StateManager;
use actix_web::{post, web, HttpResponse};

/// Begins the oauth login flow
#[utoipa::path(responses((status = 200, body = String)))]
#[post("/oauth2/init")]
pub async fn oauth_init(state: web::Data<StateManager>) -> HttpResponse {
    let service = state.get_oauth_service();
    let redirect_uri = service.init_oauth();
    HttpResponse::Ok().body(redirect_uri)
}
