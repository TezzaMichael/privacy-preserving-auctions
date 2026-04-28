use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::models::{
    auction::Auction,
    bulletin_board::BulletinEntry,
    enums::{AuctionPhase, ProofStatus},
};

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub username: String,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AuctionResponse {
    pub auction: Auction,
    pub phase: AuctionPhase,
    pub bid_count: usize,
}

#[derive(Debug, Serialize)]
pub struct SubmitBidResponse {
    pub bid_id: Uuid,
    pub bb_entry_id: Uuid,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RevealResponse {
    pub winner_id: Uuid,
    pub winner_value: u64,
    pub bb_entry_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct SubmitProofResponse {
    pub proof_id: Uuid,
    pub bb_entry_id: Uuid,
    pub status: ProofStatus,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BulletinBoardResponse {
    pub auction_id: Uuid,
    pub entries: Vec<BulletinEntry>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub auction_id: Uuid,
    pub chain_valid: bool,
    pub winner_proof_valid: bool,
    pub loser_proofs_valid: usize,
    pub loser_proofs_invalid: usize,
    pub errors: Vec<String>,
}