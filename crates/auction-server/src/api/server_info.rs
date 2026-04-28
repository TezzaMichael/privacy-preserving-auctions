use std::sync::Arc;
use axum::{extract::State, routing::get, Json, Router};
use auction_core::responses::ServerPublicKeyResponse;
use crate::{errors::ApiResult, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/server/public-key", get(server_public_key))
}

async fn server_public_key(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ServerPublicKeyResponse>> {
    Ok(Json(ServerPublicKeyResponse {
        public_key_hex: state.server_verifier.to_hex(),
        pedersen_g_hex: hex::encode(state.pedersen_generators.g.compress().to_bytes()),
        pedersen_h_hex: hex::encode(state.pedersen_generators.h.compress().to_bytes()),
    }))
}