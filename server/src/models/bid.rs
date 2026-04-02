use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bid {
    pub id: u64,
    pub user_id: u64,
    pub auction_id: u64,
    pub commitment: String,
    pub timestamp: u64,
}