use sqlx::SqlitePool;
use uuid::Uuid;
use auction_core::{errors::AuctionError, proof::{LoserProofRecord, WinnerRevealRecord}};

pub struct ProofRepo(pub SqlitePool);

impl ProofRepo {
    pub async fn insert_winner_reveal(&self, r: &WinnerRevealRecord) -> Result<(), AuctionError> {
        sqlx::query!(
            "INSERT INTO winner_reveals (id, auction_id, winner_id, bid_id, revealed_value,
             proof_json, bb_sequence, submitted_at)
             VALUES (?,?,?,?,?,?,?,?)",
            r.id, r.auction_id, r.winner_id, r.bid_id, r.revealed_value,
            r.proof_json, r.bb_sequence, r.submitted_at
        )
        .execute(&self.0)
        .await?;
        Ok(())
    }

    pub async fn find_winner_reveal(&self, auction_id: Uuid) -> Result<Option<WinnerRevealRecord>, AuctionError> {
        sqlx::query_as!(
            WinnerRevealRecord,
            r#"SELECT id as "id!: Uuid", auction_id as "auction_id!: Uuid",
               winner_id as "winner_id!: Uuid", bid_id as "bid_id!: Uuid",
               revealed_value, proof_json, bb_sequence,
               submitted_at as "submitted_at: _"
               FROM winner_reveals WHERE auction_id = ?"#,
            auction_id
        )
        .fetch_optional(&self.0)
        .await
        .map_err(AuctionError::Storage)
    }

    pub async fn update_winner_bb_sequence(&self, id: Uuid, seq: i64) -> Result<(), AuctionError> {
        sqlx::query!("UPDATE winner_reveals SET bb_sequence = ? WHERE id = ?", seq, id)
            .execute(&self.0)
            .await?;
        Ok(())
    }

    pub async fn insert_loser_proof(&self, p: &LoserProofRecord) -> Result<(), AuctionError> {
        sqlx::query!(
            "INSERT INTO loser_proofs (id, auction_id, bidder_id, bid_id, revealed_value,
             proof_json, verified, bb_sequence, submitted_at)
             VALUES (?,?,?,?,?,?,?,?,?)",
            p.id, p.auction_id, p.bidder_id, p.bid_id, p.revealed_value,
            p.proof_json, p.verified, p.bb_sequence, p.submitted_at
        )
        .execute(&self.0)
        .await?;
        Ok(())
    }

    pub async fn find_loser_proofs(&self, auction_id: Uuid) -> Result<Vec<LoserProofRecord>, AuctionError> {
        sqlx::query_as!(
            LoserProofRecord,
            r#"SELECT id as "id!: Uuid", auction_id as "auction_id!: Uuid",
            bidder_id as "bidder_id!: Uuid", bid_id as "bid_id!: Uuid",
            revealed_value, proof_json, CAST(verified AS BOOLEAN) as "verified!: bool", bb_sequence,
            submitted_at as "submitted_at!: _"
            FROM loser_proofs WHERE auction_id = ? ORDER BY submitted_at ASC"#,
            auction_id
        )
        .fetch_all(&self.0)
        .await
        .map_err(AuctionError::Storage)
    }

    pub async fn find_loser_proof_by_bidder(
        &self, auction_id: Uuid, bidder_id: Uuid,
    ) -> Result<Option<LoserProofRecord>, AuctionError> {
        sqlx::query_as!(
            LoserProofRecord,
            r#"SELECT id as "id!: Uuid", auction_id as "auction_id!: Uuid",
            bidder_id as "bidder_id!: Uuid", bid_id as "bid_id!: Uuid",
            revealed_value, proof_json, CAST(verified AS BOOLEAN) as "verified!: bool", bb_sequence,
            submitted_at as "submitted_at!: _"
            FROM loser_proofs WHERE auction_id = ? AND bidder_id = ?"#,
            auction_id, bidder_id
        )
        .fetch_optional(&self.0)
        .await
        .map_err(AuctionError::Storage)
    }

    pub async fn mark_loser_proof_verified(&self, id: Uuid) -> Result<(), AuctionError> {
        sqlx::query!("UPDATE loser_proofs SET verified = TRUE WHERE id = ?", id)
            .execute(&self.0)
            .await?;
        Ok(())
    }

    pub async fn update_loser_bb_sequence(&self, id: Uuid, seq: i64) -> Result<(), AuctionError> {
        sqlx::query!("UPDATE loser_proofs SET bb_sequence = ? WHERE id = ?", seq, id)
            .execute(&self.0)
            .await?;
        Ok(())
    }
}