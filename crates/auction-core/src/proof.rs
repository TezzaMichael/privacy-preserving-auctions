use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct WinnerRevealRecord {
    pub id: Uuid,
    pub auction_id: Uuid,
    pub winner_id: Uuid,
    pub bid_id: Uuid,
    pub revealed_value: i64,
    pub proof_json: String,
    pub bb_sequence: Option<i64>,
    pub submitted_at: DateTime<Utc>,
}

impl WinnerRevealRecord {
    pub fn new(auction_id: Uuid, winner_id: Uuid, bid_id: Uuid, revealed_value: i64, proof_json: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            auction_id,
            winner_id,
            bid_id,
            revealed_value,
            proof_json,
            bb_sequence: None,
            submitted_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct LoserProofRecord {
    pub id: Uuid,
    pub auction_id: Uuid,
    pub bidder_id: Uuid,
    pub bid_id: Uuid,
    pub revealed_value: i64,
    pub proof_json: String,
    pub verified: bool,
    pub bb_sequence: Option<i64>,
    pub submitted_at: DateTime<Utc>,
}

impl LoserProofRecord {
    pub fn new(auction_id: Uuid, bidder_id: Uuid, bid_id: Uuid, revealed_value: i64, proof_json: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            auction_id,
            bidder_id,
            bid_id,
            revealed_value,
            proof_json,
            verified: false,
            bb_sequence: None,
            submitted_at: Utc::now(),
        }
    }
}