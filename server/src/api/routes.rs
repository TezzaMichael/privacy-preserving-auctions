use axum::{
    routing::{get, post},
    Router,
};

use super::register::register_user;
use super::auction::create_auction;
use super::bid::submit_bid;

pub fn create_router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/register", post(register_user))
        .route("/auction", post(create_auction))
        .route("/bid", post(submit_bid))
}

async fn root() -> &'static str {
    "Privacy-Preserving Auction Server"
}