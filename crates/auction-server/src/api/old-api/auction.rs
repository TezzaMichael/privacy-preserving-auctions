use axum::Json;
use serde::Deserialize;

use crate::models::auction::Auction;

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
pub struct CreateAuctionRequest {
    pub id: u64,
    pub min_bid: u64,
    pub max_bid: u64,
    pub step: u64,
    pub duration_seconds: u64,
}

pub async fn create_auction(
    Json(payload): Json<CreateAuctionRequest>,
) -> Json<Auction> {

    let start_time = current_timestamp();

    let auction = Auction {
        id: payload.id,
        min_bid: payload.min_bid,
        max_bid: payload.max_bid,
        step: payload.step,
        start_time,
        end_time: start_time + payload.duration_seconds,
        bids: vec![],
        winner: None,
        winning_price: None,
    };

    Json(auction)
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}