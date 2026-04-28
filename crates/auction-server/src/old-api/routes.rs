use axum::{
    routing::{get, post},
    Router,
};
use std::sync::{Arc, Mutex};
use crate::models::bulletin_board::BulletinBoard;

use super::register::register_user;
use super::auction::create_auction;
use super::bid::submit_bid;

pub fn create_router() -> Router<Arc<Mutex<BulletinBoard>>> {
    Router::new()
        .route("/", get(root))
        .route("/register", post(register_user))
        .route("/auction", post(create_auction))
        .route("/bid", post(submit_bid))
}

async fn root() -> &'static str {
    "Privacy-Preserving Auction Server"
}