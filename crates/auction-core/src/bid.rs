use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SealedBid {
    pub id: Uuid,
    pub auction_id: Uuid,
    pub bidder_id: Uuid,
    pub commitment_hex: String,
    pub bidder_signature_hex: String,
    pub bb_sequence: Option<i64>,
    pub submitted_at: DateTime<Utc>,
}

impl SealedBid {
    pub fn new(auction_id: Uuid, bidder_id: Uuid, commitment_hex: String, bidder_signature_hex: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            auction_id,
            bidder_id,
            commitment_hex,
            bidder_signature_hex,
            bb_sequence: None,
            submitted_at: Utc::now(),
        }
    }
}