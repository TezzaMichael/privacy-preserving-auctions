use axum::Json;
use serde::Deserialize;

use crate::models::bid::Bid;

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
pub struct SubmitBidRequest {
    pub id: u64,
    pub auction_id: u64,
    pub user_id: u64,
    pub commitment: String,
}

pub async fn submit_bid(
    Json(payload): Json<SubmitBidRequest>,
) -> Json<Bid> {

    let bid = Bid {
        id: payload.id,
        auction_id: payload.auction_id,
        user_id: payload.user_id,
        commitment: payload.commitment,
        timestamp: current_timestamp(),
    };

    Json(bid)
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}