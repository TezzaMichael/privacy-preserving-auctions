use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::Type))]
#[cfg_attr(not(target_arch = "wasm32"), sqlx(type_name = "TEXT"))]
pub enum EntryKind {
    AuctionCreate,
    AuctionOpen,
    SealedBid,
    AuctionClose,
    WinnerReveal,
    LoserProof,
    ProofCertificate,
    AuctionFinalize,
}

impl std::fmt::Display for EntryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct BulletinBoardEntry {
    pub sequence: i64,
    pub auction_id: Uuid,
    pub entry_kind: EntryKind,
    pub payload_json: String,
    pub prev_hash_hex: String,
    pub entry_hash_hex: String,
    pub server_signature_hex: String,
    pub recorded_at: DateTime<Utc>,
}

impl BulletinBoardEntry {
    pub fn prev_hash_bytes(&self) -> Option<[u8; 32]> { hex_to_32(&self.prev_hash_hex) }
    pub fn entry_hash_bytes(&self) -> Option<[u8; 32]> { hex_to_32(&self.entry_hash_hex) }
}

fn hex_to_32(s: &str) -> Option<[u8; 32]> {
    hex::decode(s).ok()?.try_into().ok()
}