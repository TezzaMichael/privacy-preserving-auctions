use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::enums::AuctionPhase;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auction {
    pub id: Uuid,
    pub name: String,
    pub description: String,

    pub min_bid: u64,
    pub max_bid: u64,
    pub step: u64,

    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,

    pub claim_secs: u64,
    pub proof_secs: u64,

    pub winner_id: Option<Uuid>,
    pub winning_price: Option<u64>,
    pub winner_commitment: Option<String>,
    pub winner_blinding: Option<String>,

    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

impl Auction {
    pub fn phase(&self) -> AuctionPhase {
        let now = Utc::now();
        let claim_end =
            self.end_time + chrono::Duration::seconds(self.claim_secs as i64);
        let proof_end =
            claim_end + chrono::Duration::seconds(self.proof_secs as i64);
        match () {
            _ if now < self.start_time =>
                AuctionPhase::Pending,
            _ if now <= self.end_time =>
                AuctionPhase::BiddingOpen,
            _ if now <= claim_end =>
                AuctionPhase::ClaimPhase,
            _ if now <= proof_end =>
                AuctionPhase::ProofPhase,
            _ =>
                AuctionPhase::Closed,
        }
    }

    pub fn validate_bid(&self, value: u64) -> Result<(), &'static str> {
        if value < self.min_bid {
            return Err("offerta sotto il minimo");
        }
        if value > self.max_bid {
            return Err("offerta sopra il massimo");
        }
        if (value - self.min_bid) % self.step != 0 {
            return Err("offerta non allineata allo step");
        }

        Ok(())
    }
}