use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "PascalCase")]
pub enum AuctionStatus {
    Pending,
    BiddingOpen,
    ClaimPhase,
    ProofPhase,
    Closed,
}

impl AuctionStatus {
    pub fn can_transition_to(&self, next: &AuctionStatus) -> bool {
        matches!(
            (self, next),
            (Self::Pending, Self::BiddingOpen)
            | (Self::BiddingOpen, Self::ClaimPhase)
            | (Self::ClaimPhase, Self::ProofPhase)
            | (Self::ProofPhase, Self::Closed)
        )
    }
}

impl std::fmt::Display for AuctionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending     => write!(f, "Pending"),
            Self::BiddingOpen => write!(f, "BiddingOpen"),
            Self::ClaimPhase  => write!(f, "ClaimPhase"),
            Self::ProofPhase  => write!(f, "ProofPhase"),
            Self::Closed      => write!(f, "Closed"),
        }
    }
}