use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::enums::AuctionStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct Auction {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub title: String,
    pub description: String,
    pub status: AuctionStatus,
    pub server_signature_hex: Option<String>,
    pub bb_create_sequence: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub min_bid: i64,
    pub max_bid: Option<i64>,
    pub bid_step: i64,
    pub end_time: DateTime<Utc>,
}

impl Auction {
    pub fn new(creator_id: Uuid, title: String, description: String,  min_bid: i64, max_bid: Option<i64>, bid_step: i64, duration_seconds: i64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            creator_id,
            title,
            description,
            status: AuctionStatus::Pending,
            server_signature_hex: None,
            bb_create_sequence: None,
            created_at: now,
            updated_at: now,
            min_bid,
            max_bid,
            bid_step,
            end_time: now + chrono::Duration::seconds(duration_seconds),
        }
    }
}