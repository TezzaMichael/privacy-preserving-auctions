use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub public_key_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuctionRequest {
    pub title: String,
    pub description: String,
    pub min_bid: u64,
    pub max_bid: Option<u64>,
    pub step: u64,
    pub duration_seconds: i64, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitBidRequest {
    pub commitment_hex: String,
    pub bidder_signature_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevealWinnerRequest {
    pub bid_id: Uuid,
    pub revealed_value: u64,
    pub proof_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitLoserProofRequest {
    pub bid_id: Uuid,
    pub revealed_value: u64,
    pub proof_json: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VerifyCommitmentRequest {
    pub commitment_hex: String,
    pub value: u64,
    pub blinding_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyProofRequest {
    pub proof_json: String,
}