use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{
    auction::Auction,
    bid::SealedBid,
    bulletin_board::BulletinBoardEntry,
    enums::AuctionStatus,
    proof::{LoserProofRecord, WinnerRevealRecord},
    user::User,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub username: String,
    pub public_key_hex: String,
}

impl From<User> for RegisterResponse {
    fn from(u: User) -> Self {
        Self { user_id: u.id, username: u.username, public_key_hex: u.public_key_hex }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub jwt_token: String,
    pub user_id: Uuid,
    pub username: String,
    pub public_key_hex: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeResponse {
    pub user_id: Uuid,
    pub username: String,
    pub public_key_hex: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for MeResponse {
    fn from(u: User) -> Self {
        Self { user_id: u.id, username: u.username, public_key_hex: u.public_key_hex, created_at: u.created_at }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuctionResponse {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub title: String,
    pub description: String,
    pub status: AuctionStatus,
    pub reserve_price: Option<i64>,
    pub bb_create_sequence: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Auction> for AuctionResponse {
    fn from(a: Auction) -> Self {
        Self {
            id: a.id,
            creator_id: a.creator_id,
            title: a.title,
            description: a.description,
            status: a.status,
            reserve_price: a.reserve_price,
            bb_create_sequence: a.bb_create_sequence,
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuctionListResponse {
    pub auctions: Vec<AuctionResponse>,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitBidResponse {
    pub bid_id: Uuid,
    pub bb_entry_hash_hex: String,
    pub bb_sequence: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SealedBidResponse {
    pub bid_id: Uuid,
    pub bidder_id: Uuid,
    pub commitment_hex: String,
    pub bidder_signature_hex: String,
    pub bb_sequence: Option<i64>,
    pub submitted_at: DateTime<Utc>,
}

impl From<SealedBid> for SealedBidResponse {
    fn from(b: SealedBid) -> Self {
        Self {
            bid_id: b.id,
            bidder_id: b.bidder_id,
            commitment_hex: b.commitment_hex,
            bidder_signature_hex: b.bidder_signature_hex,
            bb_sequence: b.bb_sequence,
            submitted_at: b.submitted_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BidListResponse {
    pub bids: Vec<SealedBidResponse>,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevealWinnerResponse {
    pub reveal_id: Uuid,
    pub winner_id: Uuid,
    pub revealed_value: i64,
    pub bb_entry_hash_hex: String,
    pub bb_sequence: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WinnerRevealDetailResponse {
    pub reveal_id: Uuid,
    pub auction_id: Uuid,
    pub winner_id: Uuid,
    pub bid_id: Uuid,
    pub revealed_value: i64,
    pub proof_json: String,
    pub bb_sequence: Option<i64>,
    pub submitted_at: DateTime<Utc>,
}

impl From<WinnerRevealRecord> for WinnerRevealDetailResponse {
    fn from(r: WinnerRevealRecord) -> Self {
        Self {
            reveal_id: r.id,
            auction_id: r.auction_id,
            winner_id: r.winner_id,
            bid_id: r.bid_id,
            revealed_value: r.revealed_value,
            proof_json: r.proof_json,
            bb_sequence: r.bb_sequence,
            submitted_at: r.submitted_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoserProofResponse {
    pub proof_id: Uuid,
    pub bidder_id: Uuid,
    pub bid_id: Uuid,
    pub revealed_value: i64,
    pub proof_json: String,
    pub verified: bool,
    pub bb_sequence: Option<i64>,
    pub submitted_at: DateTime<Utc>,
}

impl From<LoserProofRecord> for LoserProofResponse {
    fn from(p: LoserProofRecord) -> Self {
        Self {
            proof_id: p.id,
            bidder_id: p.bidder_id,
            bid_id: p.bid_id,
            revealed_value: p.revealed_value,
            proof_json: p.proof_json,
            verified: p.verified,
            bb_sequence: p.bb_sequence,
            submitted_at: p.submitted_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoserProofListResponse {
    pub proofs: Vec<LoserProofResponse>,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulletinBoardResponse {
    pub auction_id: Uuid,
    pub entries: Vec<BulletinBoardEntry>,
    pub total: usize,
    pub head_hash_hex: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulletinBoardEntryResponse {
    pub entry: BulletinBoardEntry,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyCommitmentResponse {
    pub valid: bool,
    pub commitment_hex: String,
    pub value: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyProofResponse {
    pub valid: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptVerificationResponse {
    pub auction_id: Uuid,
    pub chain_integrity_valid: bool,
    pub winner_proof_valid: bool,
    pub all_loser_proofs_valid: bool,
    pub server_signatures_valid: bool,
    pub commitments_consistent: bool,
    pub fully_valid: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerPublicKeyResponse {
    pub public_key_hex: String,
    pub pedersen_g_hex: String,
    pub pedersen_h_hex: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, code: u16) -> Self {
        Self { error: error.into(), code }
    }
}