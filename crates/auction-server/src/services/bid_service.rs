use sqlx::SqlitePool;
use uuid::Uuid;
use auction_core::{bid::SealedBid, enums::AuctionStatus, errors::AuctionError};
use auction_crypto::keys::verify_commitment_signature;
use crate::storage::bid_repo::BidRepo;

pub struct BidService {
    repo: BidRepo,
}

impl BidService {
    pub fn new(pool: SqlitePool) -> Self { Self { repo: BidRepo(pool) } }

    pub async fn submit(
        &self,
        auction_id: Uuid,
        bidder_id: Uuid,
        bidder_public_key_hex: &str,
        commitment_hex: String,
        bidder_signature_hex: String,
    ) -> Result<SealedBid, AuctionError> {
        if self.repo.find_by_bidder_and_auction(bidder_id, auction_id).await?.is_some() {
            return Err(AuctionError::DuplicateBid);
        }
        let sig_bytes = hex::decode(&bidder_signature_hex)
            .map_err(|_| AuctionError::InvalidBidderSignature)?;
        if !verify_commitment_signature(
            bidder_public_key_hex,
            auction_id.as_bytes(),
            &commitment_hex,
            &sig_bytes,
        ) {
            return Err(AuctionError::InvalidBidderSignature);
        }
        let bid = SealedBid::new(auction_id, bidder_id, commitment_hex, bidder_signature_hex);
        self.repo.insert(&bid).await?;
        Ok(bid)
    }

    pub async fn list_by_auction(&self, auction_id: Uuid) -> Result<Vec<SealedBid>, AuctionError> {
        self.repo.find_by_auction(auction_id).await
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<SealedBid, AuctionError> {
        self.repo.find_by_id(id).await
    }

    pub async fn find_by_bidder_and_auction(
        &self, bidder_id: Uuid, auction_id: Uuid,
    ) -> Result<Option<SealedBid>, AuctionError> {
        self.repo.find_by_bidder_and_auction(bidder_id, auction_id).await
    }

    pub async fn update_bb_sequence(&self, id: Uuid, seq: i64) -> Result<(), AuctionError> {
        self.repo.update_bb_sequence(id, seq).await
    }

    pub async fn count_by_auction(&self, auction_id: Uuid) -> Result<i64, AuctionError> {
        self.repo.count_by_auction(auction_id).await
    }
}