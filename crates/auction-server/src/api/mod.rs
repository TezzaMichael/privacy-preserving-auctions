use std::sync::Arc;
use axum::Router;
use crate::state::AppState;

pub mod auctions;
pub mod auth;
pub mod bids;
pub mod bulletin_board;
pub mod proofs;
pub mod server_info;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .merge(auth::router())
        .merge(auctions::router())
        .merge(bids::router())
        .merge(proofs::router())
        .merge(bulletin_board::router())
        .merge(server_info::router())
}