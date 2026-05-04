use axum::{Json, extract::State};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

use crate::models::bid::Bid;
use crate::models::bulletin_board::BulletinBoard;
use crate::services::bulletin_board;

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
pub struct SubmitBidRequest {
    pub auction_id: u64,
    pub commitment: String,
    pub user_id: u64, // This should come from the authenticated user context in a real application
}

pub async fn submit_bid(
    State(board): State<Arc<Mutex<BulletinBoard>>>,
    Json(payload): Json<SubmitBidRequest>,
) -> Json<Bid> {

    let mut board = board.lock().unwrap();

    // generate a new bid ID
    let bid_id = board.bids.len() as u64 + 1;

    let bid = Bid {
        id: bid_id,
        user_id: payload.user_id, // This should come from the authenticated user context in a real application
        auction_id: payload.auction_id,
        commitment: payload.commitment,
        timestamp: current_timestamp(),
    };

    bulletin_board::submit_bid(&mut board, bid.clone());

    Json(bid)
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}