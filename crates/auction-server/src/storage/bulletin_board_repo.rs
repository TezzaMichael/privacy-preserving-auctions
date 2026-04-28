use sqlx::SqlitePool;
use uuid::Uuid;
use auction_core::{bulletin_board::{BulletinBoardEntry, EntryKind}, errors::AuctionError};

pub struct BulletinBoardRepo(pub SqlitePool);

impl BulletinBoardRepo {
    pub async fn insert(&self, e: &BulletinBoardEntry) -> Result<(), AuctionError> {
        let kind = e.entry_kind.to_string();
        sqlx::query!(
            "INSERT INTO bulletin_board (sequence, auction_id, entry_kind, payload_json,
             prev_hash_hex, entry_hash_hex, server_signature_hex, recorded_at)
             VALUES (?,?,?,?,?,?,?,?)",
            e.sequence, e.auction_id, kind, e.payload_json,
            e.prev_hash_hex, e.entry_hash_hex, e.server_signature_hex, e.recorded_at
        )
        .execute(&self.0)
        .await?;
        Ok(())
    }

    pub async fn find_by_auction(&self, auction_id: Uuid) -> Result<Vec<BulletinBoardEntry>, AuctionError> {
        sqlx::query_as!(
            BulletinBoardEntry,
            r#"SELECT sequence, auction_id as "auction_id: Uuid",
               entry_kind as "entry_kind: EntryKind", payload_json,
               prev_hash_hex, entry_hash_hex, server_signature_hex,
               recorded_at as "recorded_at: _"
               FROM bulletin_board WHERE auction_id = ? ORDER BY sequence ASC"#,
            auction_id
        )
        .fetch_all(&self.0)
        .await
        .map_err(AuctionError::Storage)
    }

    pub async fn find_by_sequence(
        &self, auction_id: Uuid, sequence: i64,
    ) -> Result<Option<BulletinBoardEntry>, AuctionError> {
        sqlx::query_as!(
            BulletinBoardEntry,
            r#"SELECT sequence, auction_id as "auction_id: Uuid",
               entry_kind as "entry_kind: EntryKind", payload_json,
               prev_hash_hex, entry_hash_hex, server_signature_hex,
               recorded_at as "recorded_at: _"
               FROM bulletin_board WHERE auction_id = ? AND sequence = ?"#,
            auction_id, sequence
        )
        .fetch_optional(&self.0)
        .await
        .map_err(AuctionError::Storage)
    }

    pub async fn get_head(&self, auction_id: Uuid) -> Result<Option<BulletinBoardEntry>, AuctionError> {
        sqlx::query_as!(
            BulletinBoardEntry,
            r#"SELECT sequence, auction_id as "auction_id: Uuid",
               entry_kind as "entry_kind: EntryKind", payload_json,
               prev_hash_hex, entry_hash_hex, server_signature_hex,
               recorded_at as "recorded_at: _"
               FROM bulletin_board WHERE auction_id = ?
               ORDER BY sequence DESC LIMIT 1"#,
            auction_id
        )
        .fetch_optional(&self.0)
        .await
        .map_err(AuctionError::Storage)
    }

    pub async fn next_sequence(&self, auction_id: Uuid) -> Result<i64, AuctionError> {
        let row = sqlx::query!(
            "SELECT COUNT(*) as cnt FROM bulletin_board WHERE auction_id = ?", auction_id
        )
        .fetch_one(&self.0)
        .await?;
        Ok(row.cnt as i64)
    }
}