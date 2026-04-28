use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::models::proof::SchnorrProof;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub public_key: String,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAuctionRequest {
    pub name: String,
    pub description: String,
    pub min_bid: u64,
    pub max_bid: u64,
    pub step: u64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub claim_secs: Option<u64>,
    pub proof_secs: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitBidRequest {
    pub commitment: String,
    pub signature: String,
}

#[derive(Debug, Deserialize)]
pub struct RevealBidRequest {
    pub value: u64,
    pub blinding: String,
    pub schnorr_proof: SchnorrProof,
}

#[derive(Debug, Deserialize)]
pub struct SubmitProofRequest {
    pub proof_bytes: String,
    pub v_commitment: String,
    pub challenge: String,
}