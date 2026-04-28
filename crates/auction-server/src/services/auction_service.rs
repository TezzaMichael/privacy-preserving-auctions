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
        reserve_price: Option<i64>,
    ) -> Result<Auction, AuctionError> {
        let auction = Auction::new(creator_id, title, description, reserve_price);
        self.repo.insert(&auction).await?;
        Ok(auction)
    }

    pub async fn get(&self, id: Uuid) -> Result<Auction, AuctionError> {
        self.repo.find_by_id(id).await
    }

    pub async fn list(&self) -> Result<Vec<Auction>, AuctionError> {
        self.repo.list_all().await
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