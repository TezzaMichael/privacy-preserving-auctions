use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::enums::ProofStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchnorrProof {
    pub big_r: String,
    pub challenge: String,
    pub s_value: String,
    pub s_blinding: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofCertificate {
    pub id: Uuid,
    pub auction_id: Uuid,
    pub bidder_id: Uuid,
    pub loser_commitment: String,
    pub winner_commitment: String,
    pub proof_bytes: String,
    pub v_commitment: String,
    pub challenge: String,
    pub submitted_at: DateTime<Utc>,
    pub status: ProofStatus,
    pub error: Option<String>,
}