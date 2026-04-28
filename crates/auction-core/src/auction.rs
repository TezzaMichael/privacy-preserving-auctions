use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::enums::AuctionStatus;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Auction {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub title: String,
    pub description: String,
    pub status: AuctionStatus,
    pub reserve_price: Option<i64>,
    pub server_signature_hex: Option<String>,
    pub bb_create_sequence: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Auction {
    pub fn new(creator_id: Uuid, title: String, description: String, reserve_price: Option<i64>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            creator_id,
            title,
            description,
            status: AuctionStatus::Pending,
            reserve_price,
            server_signature_hex: None,
            bb_create_sequence: None,
            created_at: now,
            updated_at: now,
        }
    }
}