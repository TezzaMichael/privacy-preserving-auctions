use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::proof::{LoserProofRecord, WinnerRevealRecord};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionRevealState {
    pub auction_id: Uuid,
    pub winner_reveal: Option<WinnerRevealRecord>,
    pub loser_proofs: Vec<LoserProofRecord>,
    pub total_bids: usize,
}

impl AuctionRevealState {
    pub fn new(auction_id: Uuid, total_bids: usize) -> Self {
        Self { auction_id, winner_reveal: None, loser_proofs: Vec::new(), total_bids }
    }

    pub fn is_complete(&self) -> bool {
        self.winner_reveal.is_some()
            && self.loser_proofs.len() >= self.total_bids.saturating_sub(1)
    }

    pub fn winner_value(&self) -> Option<i64> {
        self.winner_reveal.as_ref().map(|r| r.revealed_value)
    }

    pub fn all_loser_proofs_verified(&self) -> bool {
        self.loser_proofs.iter().all(|p| p.verified)
    }
}