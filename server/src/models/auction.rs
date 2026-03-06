use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auction {
    pub id: u64,
    pub min_bid: u64,     // T1
    pub max_bid: u64,     // T2
    pub step: u64,        // S
    pub start_time: u64,
    pub end_time: u64,
    pub bids: Vec<u64>,   // IDs delle bid
    pub winner: Option<u64>, // user_id del vincitore
    pub winning_price: Option<u64>,
}