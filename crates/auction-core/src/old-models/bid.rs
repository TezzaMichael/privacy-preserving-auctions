use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::enums::BidStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bid {

    pub id: Uuid,
    pub auction_id: Uuid,
    pub user_id: Uuid,
    pub commitment: String,
    pub signature: String,
    pub submitted_at: DateTime<Utc>,
    pub status: BidStatus,
}