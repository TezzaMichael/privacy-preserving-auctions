use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuctionCreatePayload {
    pub auction_id: Uuid,
    pub creator_id: Uuid,
    pub title: String,
    pub reserve_price: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuctionStatusPayload {
    pub auction_id: Uuid,
    pub new_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SealedBidPayload {
    pub bid_id: Uuid,
    pub auction_id: Uuid,
    pub bidder_id: Uuid,
    pub commitment_hex: String,
    pub bidder_signature_hex: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WinnerRevealPayload {
    pub reveal_id: Uuid,
    pub auction_id: Uuid,
    pub winner_id: Uuid,
    pub bid_id: Uuid,
    pub revealed_value: i64,
    pub proof_json: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoserProofPayload {
    pub proof_id: Uuid,
    pub auction_id: Uuid,
    pub bidder_id: Uuid,
    pub bid_id: Uuid,
    pub revealed_value: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuctionFinalizePayload {
    pub auction_id: Uuid,
    pub winner_id: Uuid,
    pub winner_value: i64,
    pub total_bids: i64,
}