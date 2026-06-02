use sqlx::SqlitePool;
use uuid::Uuid;
use auction_core::{auction::Auction, enums::AuctionStatus, errors::AuctionError};
use crate::storage::auction_repo::AuctionRepo;

pub struct AuctionService {
    repo: AuctionRepo,
}

impl AuctionService {
    pub fn new(pool: SqlitePool) -> Self { Self { repo: AuctionRepo(pool) } }

    pub async fn create(
        &self,
        creator_id: Uuid,
        title: String,
        description: String,
        min_bid: i64,
        max_bid: i64, 
        bid_step: i64,
        duration_seconds: i64,
    ) -> Result<Auction, AuctionError> {
        
        if bid_step <= 0 {
            return Err(AuctionError::Internal("bid_step must be a positive integer".into()));
        }

        // Logic checks
        if min_bid > max_bid {
            return Err(AuctionError::Internal("min_bid cannot be greater than max_bid".into()));
        }
        if bid_step > max_bid {
            return Err(AuctionError::Internal("bid_step cannot be greater than max_bid".into()));
        }
        if bid_step > (max_bid - min_bid) && min_bid != max_bid {
            return Err(AuctionError::Internal("bid_step is too large relative to the range between min_bid and max_bid".into()));
        }

        let mut auction = Auction::new(creator_id, title, description, min_bid, Some(max_bid), bid_step, duration_seconds);
        
        auction.status = AuctionStatus::BiddingOpen;

        self.repo.insert(&auction).await?;
        Ok(auction)
    }

    pub async fn get(&self, id: Uuid) -> Result<Auction, AuctionError> {
        self.repo.find_by_id(id).await
    }

    pub async fn list(&self) -> Result<Vec<Auction>, AuctionError> {
        self.repo.list_all().await
    }

    pub async fn system_transition(
        &self,
        id: Uuid,
        to: AuctionStatus,
    ) -> Result<Auction, AuctionError> {
        let auction = self.repo.find_by_id(id).await?;
        if !auction.status.can_transition_to(&to) {
            return Err(AuctionError::InvalidStateTransition {
                from: auction.status,
                to,
            });
        }
        self.repo.update_status(id, &to).await?;
        self.repo.find_by_id(id).await
    }

    pub async fn transition(
    &self,
    id: Uuid,
    requester_id: Uuid,
    to: AuctionStatus,
) -> Result<Auction, AuctionError> {
    let auction = self.repo.find_by_id(id).await?;

    if auction.creator_id != requester_id {
        return Err(AuctionError::NotCreator);
    }

    // time-locked transition: only allow moving to ClaimPhase after end_time
    if to == AuctionStatus::ClaimPhase && chrono::Utc::now() < auction.end_time {
        return Err(AuctionError::Internal("Auction cannot be claimed before it ends".into()));
    }

    if !auction.status.can_transition_to(&to) {
        return Err(AuctionError::InvalidStateTransition {
            from: auction.status,
            to,
        });
    }

    self.repo.update_status(id, &to).await?;
    self.repo.find_by_id(id).await
}

    pub async fn set_bb_sequence(&self, id: Uuid, seq: i64) -> Result<(), AuctionError> {
        self.repo.update_bb_sequence(id, seq).await
    }

    pub async fn set_server_signature(&self, id: Uuid, sig_hex: &str) -> Result<(), AuctionError> {
        self.repo.update_server_signature(id, sig_hex).await
    }

    pub async fn require_status(&self, id: Uuid, required: &AuctionStatus) -> Result<Auction, AuctionError> {
        let auction = self.repo.find_by_id(id).await?;
        if &auction.status != required {
            return Err(AuctionError::WrongState {
                current: auction.status,
                required: required.clone(),
            });
        }
        Ok(auction)
    }

}