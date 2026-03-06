use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofCertificate {
    pub bidder_id: u64,
    pub auction_id: u64,
    pub challenge: String,
    pub response: String,
}