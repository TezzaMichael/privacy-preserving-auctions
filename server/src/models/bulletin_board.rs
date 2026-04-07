use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::enums::EntryKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulletinEntry {
    pub id: Uuid,
    pub sequence: u64,
    pub kind: EntryKind,
    pub auction_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
    pub prev_hash: String,
    pub entry_hash: String,
    pub server_sig: String,
}

impl BulletinEntry {
    pub fn genesis_hash() -> String {
        "0".repeat(64)
    }
}