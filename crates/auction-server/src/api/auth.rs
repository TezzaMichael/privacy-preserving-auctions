use std::sync::Arc;
use axum::{extract::State, routing::post, Json, Router};
use auction_core::{requests::{LoginRequest, RegisterRequest}, responses::{MeResponse, RegisterResponse}};
use crate::{auth::middleware::AuthUser, errors::ApiResult, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login",    post(login))
        .route("/auth/me",       axum::routing::get(me))
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> ApiResult<Json<RegisterResponse>> {
    let user = state.user_service.register(req.username, req.password, req.public_key_hex).await?;
    Ok(Json(RegisterResponse::from(user)))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> ApiResult<Json<auction_core::responses::LoginResponse>> {
    let resp = state.user_service.login(req.username, req.password).await?;
    Ok(Json(resp))
}

async fn me(
    State(state): State<Arc<AppState>>,
    AuthUser(user_id): AuthUser,
) -> ApiResult<Json<MeResponse>> {
    let user = state.user_service.get_by_id(user_id).await?;
    Ok(Json(MeResponse::from(user)))
}