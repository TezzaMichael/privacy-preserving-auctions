use sqlx::SqlitePool;
use uuid::Uuid;
use auction_core::{bid::SealedBid, errors::AuctionError};

pub struct BidRepo(pub SqlitePool);

impl BidRepo {
    pub async fn insert(&self, b: &SealedBid) -> Result<(), AuctionError> {
        sqlx::query!(
            "INSERT INTO sealed_bids (id, auction_id, bidder_id, commitment_hex,
             bidder_signature_hex, bb_sequence, submitted_at)
             VALUES (?,?,?,?,?,?,?)",
            b.id, b.auction_id, b.bidder_id, b.commitment_hex,
            b.bidder_signature_hex, b.bb_sequence, b.submitted_at
        )
        .execute(&self.0)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<SealedBid, AuctionError> {
        sqlx::query_as!(
            SealedBid,
            r#"SELECT id as "id!: Uuid", auction_id as "auction_id!: Uuid",
               bidder_id as "bidder_id!: Uuid", commitment_hex, bidder_signature_hex,
               bb_sequence, submitted_at as "submitted_at: _"
               FROM sealed_bids WHERE id = ?"#,
            id
        )
        .fetch_one(&self.0)
        .await
        .map_err(AuctionError::Storage)
    }

    pub async fn find_by_auction(&self, auction_id: Uuid) -> Result<Vec<SealedBid>, AuctionError> {
        sqlx::query_as!(
            SealedBid,
            r#"SELECT id as "id!: Uuid", auction_id as "auction_id!: Uuid",
               bidder_id as "bidder_id!: Uuid", commitment_hex, bidder_signature_hex,
               bb_sequence, submitted_at as "submitted_at: _"
               FROM sealed_bids WHERE auction_id = ? ORDER BY submitted_at ASC"#,
            auction_id
        )
        .fetch_all(&self.0)
        .await
        .map_err(AuctionError::Storage)
    }

    pub async fn find_by_bidder_and_auction(
        &self, bidder_id: Uuid, auction_id: Uuid,
    ) -> Result<Option<SealedBid>, AuctionError> {
        sqlx::query_as!(
            SealedBid,
            r#"SELECT id as "id!: Uuid", auction_id as "auction_id!: Uuid",
               bidder_id as "bidder_id!: Uuid", commitment_hex, bidder_signature_hex,
               bb_sequence, submitted_at as "submitted_at: _"
               FROM sealed_bids WHERE bidder_id = ? AND auction_id = ?"#,
            bidder_id, auction_id
        )
        .fetch_optional(&self.0)
        .await
        .map_err(AuctionError::Storage)
    }

    pub async fn update_bb_sequence(&self, id: Uuid, seq: i64) -> Result<(), AuctionError> {
        sqlx::query!("UPDATE sealed_bids SET bb_sequence = ? WHERE id = ?", seq, id)
            .execute(&self.0)
            .await?;
        Ok(())
    }

    pub async fn count_by_auction(&self, auction_id: Uuid) -> Result<i64, AuctionError> {
        let row = sqlx::query!(
            "SELECT COUNT(*) as cnt FROM sealed_bids WHERE auction_id = ?", auction_id
        )
        .fetch_one(&self.0)
        .await?;
        Ok(row.cnt.into())
    }
}