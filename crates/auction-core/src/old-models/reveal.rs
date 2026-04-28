use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::proof::SchnorrProof;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinnerReveal {
    pub id: Uuid,
    pub auction_id: Uuid,
    pub user_id: Uuid,
    pub value: u64,
    pub blinding: String,
    pub commitment: String,
    pub schnorr_proof: SchnorrProof,
    pub revealed_at: DateTime<Utc>,
}