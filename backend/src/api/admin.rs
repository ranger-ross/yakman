use crate::model::{request::CreateYakManUserPayload, YakManRole, YakManUser};
use crate::{middleware::roles::YakManRoleBinding, StateManager};
use actix_web::{
    get, put,
    web::{self, Json},
    HttpResponse,
};
use actix_web_grants::permissions::AuthDetails;
use uuid::Uuid;

/// Gets users
#[utoipa::path(responses((status = 200, body = Vec<YakManUser>)))]
#[get("/admin/v1/users")]
pub async fn get_yakman_users(
    auth_details: AuthDetails<YakManRoleBinding>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let is_admin = YakManRoleBinding::has_global_role(YakManRole::Admin, &auth_details.permissions);

    if !is_admin {
        return HttpResponse::Forbidden().finish();
    }

    let users = state.get_service().get_users().await.unwrap();

    HttpResponse::Ok().body(serde_json::to_string(&users).unwrap())
}

/// Create YakMan user
#[utoipa::path(request_body = YakManUser, responses((status = 200, body = String)))]
#[put("/admin/v1/users")]
pub async fn create_yakman_user(
    auth_details: AuthDetails<YakManRoleBinding>,
    payload: Json<CreateYakManUserPayload>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let is_admin = YakManRoleBinding::has_global_role(YakManRole::Admin, &auth_details.permissions);

    if !is_admin {
        return HttpResponse::Forbidden().finish();
    }

    let mut users = state.get_service().get_users().await.unwrap();
    let user = payload.into_inner();

    users.push(YakManUser {
        email: user.email,
        uuid: Uuid::new_v4().to_string(),
        role: user.role,
    });

    state.get_service().save_users(users).await.unwrap();

    HttpResponse::Ok().finish()
}
