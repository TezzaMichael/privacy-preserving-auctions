use sqlx::SqlitePool;
use uuid::Uuid;
use auction_core::{auction::Auction, enums::AuctionStatus, errors::AuctionError};

pub struct AuctionRepo(pub SqlitePool);

impl AuctionRepo {
    pub async fn insert(&self, a: &Auction) -> Result<(), AuctionError> {
        let status = a.status.to_string();
        sqlx::query!(
            "INSERT INTO auctions (id, creator_id, title, description, status, min_bid, max_bid, bid_step, end_time, server_signature_hex, bb_create_sequence, created_at, updated_at)
            VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)", // <-- 13 punti interrogativi
            a.id, a.creator_id, a.title, a.description, status, a.min_bid, a.max_bid, a.bid_step, a.end_time,
            a.server_signature_hex, a.bb_create_sequence, a.created_at, a.updated_at
        )
        .execute(&self.0)
        .await?;
        Ok(())
}

    pub async fn find_by_id(&self, id: Uuid) -> Result<Auction, AuctionError> {
    sqlx::query_as!(
        Auction,
        r#"SELECT id as "id!: Uuid", creator_id as "creator_id!: Uuid", title, description,
           status as "status!: AuctionStatus", min_bid, max_bid, bid_step, 
           end_time as "end_time!: _", server_signature_hex,
           bb_create_sequence, created_at as "created_at!: _", updated_at as "updated_at!: _"
           FROM auctions WHERE id = ?"#,
        id
    )
    .fetch_one(&self.0)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AuctionError::AuctionNotFound(id),
        e => AuctionError::Storage(e),
    })
}

    pub async fn list_all(&self) -> Result<Vec<Auction>, AuctionError> {
    sqlx::query_as!(
        Auction,
        r#"SELECT id as "id!: Uuid", creator_id as "creator_id!: Uuid", title, description,
           status as "status!: AuctionStatus", min_bid, max_bid, bid_step, 
           end_time as "end_time!: _", server_signature_hex,
           bb_create_sequence, created_at as "created_at!: _", updated_at as "updated_at!: _"
           FROM auctions ORDER BY created_at DESC"#
    )
    .fetch_all(&self.0)
    .await
    .map_err(AuctionError::Storage)
}

    pub async fn update_status(&self, id: Uuid, status: &AuctionStatus) -> Result<(), AuctionError> {
        let s = status.to_string();
        let now = chrono::Utc::now();
        sqlx::query!("UPDATE auctions SET status = ?, updated_at = ? WHERE id = ?", s, now, id)
            .execute(&self.0)
            .await?;
        Ok(())
    }

    pub async fn update_bb_sequence(&self, id: Uuid, seq: i64) -> Result<(), AuctionError> {
        let now = chrono::Utc::now();
        sqlx::query!(
            "UPDATE auctions SET bb_create_sequence = ?, updated_at = ? WHERE id = ?",
            seq, now, id
        )
        .execute(&self.0)
        .await?;
        Ok(())
    }

    pub async fn update_server_signature(&self, id: Uuid, sig_hex: &str) -> Result<(), AuctionError> {
        let now = chrono::Utc::now();
        sqlx::query!(
            "UPDATE auctions SET server_signature_hex = ?, updated_at = ? WHERE id = ?",
            sig_hex, now, id
        )
        .execute(&self.0)
        .await?;
        Ok(())
    }
}