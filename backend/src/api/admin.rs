use crate::StateManager;
use actix_web::{
    get, put,
    web::{self, Json},
    HttpResponse,
};
use actix_web_grants::proc_macro::has_any_role;
use yak_man_core::model::{YakManRole, YakManUser};

/// Gets users
#[utoipa::path(responses((status = 200, body = Vec<YakManUser>)))]
#[get("/admin/v1/users")]
#[has_any_role("YakManRole::Admin", type = "YakManRole")]
pub async fn get_yakman_users(state: web::Data<StateManager>) -> HttpResponse {
    let users = state.get_service().get_users().await.unwrap();

    HttpResponse::Ok().body(serde_json::to_string(&users).unwrap())
}

/// Create YakMan user
#[utoipa::path(responses((status = 200, body = String)))]
#[put("/admin/v1/users")]
#[has_any_role("YakManRole::Admin", type = "YakManRole")]
pub async fn create_yakman_user(
    user: Json<YakManUser>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let mut users = state.get_service().get_users().await.unwrap();

    users.push(user.into_inner());

    state.get_service().save_users(users).await.unwrap();

    HttpResponse::Ok().body("")
}
