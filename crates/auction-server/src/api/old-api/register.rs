use axum::Json;
use serde::Deserialize;

use crate::models::user::User;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub id: u64,
    pub username: String,
    pub password: String,
}

pub async fn register_user(
    Json(payload): Json<RegisterRequest>,
) -> Json<User> {

    let user = User {
        id: payload.id,
        username: payload.username,
        password_hash: payload.password,
    };

    Json(user)
}