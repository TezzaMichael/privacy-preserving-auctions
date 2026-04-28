use std::sync::Arc;
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
};
use uuid::Uuid;
use crate::state::AppState;

pub struct AuthUser(pub Uuid);

#[axum::async_trait]
impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or((StatusCode::UNAUTHORIZED, "missing bearer token"))?;

        let user_id = crate::auth::jwt::verify_token(auth, &state.jwt_secret)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid token"))?;

        Ok(AuthUser(user_id))
    }
}