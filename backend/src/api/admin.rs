use std::{
    fs::{self, File},
    io::Write,
};

use crate::StateManager;
use actix_web::{
    get, put,
    web::{self, Json},
    HttpResponse,
};
use serde::{Deserialize, Serialize};
use yak_man_core::model::YakManUser;

/// Gets users
#[utoipa::path(responses((status = 200, body = String)))]
#[get("/admin/v1/users")]
pub async fn get_yakman_users(state: web::Data<StateManager>) -> HttpResponse {
    let users = get_users();
    HttpResponse::Ok().body(serde_json::to_string(&users).unwrap())
}

/// Create YakMan user
#[utoipa::path(responses((status = 200, body = String)))]
#[put("/admin/v1/users")]
pub async fn create_yakman_user(
    user: Json<YakManUser>,
    state: web::Data<StateManager>,
) -> HttpResponse {
    let mut users = get_users();
    users.push(user.into_inner());

    save_users(users);

    HttpResponse::Ok().body("")
}

fn get_users() -> Vec<YakManUser> {
    let data = fs::read_to_string(get_user_datafile_path()).unwrap();

    let user_data: UsersJson = serde_json::from_str(&data).unwrap();
    return user_data.users;
}

fn save_users(users: Vec<YakManUser>) {
    let data = serde_json::to_string(&UsersJson { users: users }).unwrap();
    let data_file_path = get_user_datafile_path();
    let mut data_file = File::create(&data_file_path).unwrap();
    Write::write_all(&mut data_file, data.as_bytes()).unwrap();
}

fn get_user_datafile_path() -> String {
    "/home/ross/projects/config-manager/testing-directory/.yakman/users/users.json".to_string()
}

#[derive(Serialize, Deserialize, Debug)]
struct UsersJson {
    users: Vec<YakManUser>,
}
