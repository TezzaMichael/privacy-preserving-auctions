use thiserror::Error;
use uuid::Uuid;
use crate::enums::AuctionStatus;

#[derive(Debug, Error)]
pub enum AuctionError {
    #[error("username already taken")]
    UsernameTaken,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("user not found: {0}")]
    UserNotFound(Uuid),
    #[error("auction not found: {0}")]
    AuctionNotFound(Uuid),
    #[error("invalid state transition: {from} → {to}")]
    InvalidStateTransition { from: AuctionStatus, to: AuctionStatus },
    #[error("auction is not in required state (current: {current}, required: {required})")]
    WrongState { current: AuctionStatus, required: AuctionStatus },
    #[error("only the auction creator can perform this action")]
    NotCreator,
    #[error("bid already submitted by this bidder")]
    DuplicateBid,
    #[error("bidder signature on commitment is invalid")]
    InvalidBidderSignature,
    #[error("invalid commitment bytes")]
    InvalidCommitment,
    #[error("winner reveal already submitted")]
    RevealAlreadySubmitted,
    #[error("bidder {0} did not participate in auction {1}")]
    BidderNotInAuction(Uuid, Uuid),
    #[error("Schnorr proof of opening is invalid")]
    InvalidProof,
    #[error("revealed value does not beat winner")]
    NotALoser,
    #[error("proof already submitted for this bidder")]
    DuplicateProof,
    #[error("bulletin board integrity check failed: {0}")]
    BulletinBoardCorrupted(String),
    #[cfg(not(target_arch = "wasm32"))]
    #[error("storage error: {0}")]
    Storage(#[from] sqlx::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("internal error: {0}")]
    Internal(String),
}

impl AuctionError {
    pub fn status_code(&self) -> u16 {
        match self {
            Self::UsernameTaken                => 409,
            Self::InvalidCredentials           => 401,
            Self::UserNotFound(_)              => 404,
            Self::AuctionNotFound(_)           => 404,
            Self::NotCreator                   => 403,
            Self::WrongState { .. }            => 422,
            Self::InvalidStateTransition { .. }=> 422,
            Self::DuplicateBid                 => 409,
            Self::InvalidBidderSignature       => 400,
            Self::InvalidCommitment            => 400,
            Self::RevealAlreadySubmitted       => 409,
            Self::BidderNotInAuction(_, _)     => 404,
            Self::InvalidProof                 => 400,
            Self::NotALoser                    => 422,
            Self::DuplicateProof               => 409,
            Self::BulletinBoardCorrupted(_)    => 500,
            #[cfg(not(target_arch = "wasm32"))]
            Self::Storage(_)                   => 500,
            Self::Serialization(_)             => 500,
            Self::Internal(_)                  => 500,
        }
    }
}