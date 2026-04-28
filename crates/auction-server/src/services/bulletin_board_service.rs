use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;
use auction_core::{bulletin_board::{BulletinBoardEntry, EntryKind}, errors::AuctionError};
use auction_crypto::signature::ServerSigner;
use crate::storage::bulletin_board_repo::BulletinBoardRepo;
use sha2::{Digest, Sha256};

pub struct BulletinBoardService {
    repo: BulletinBoardRepo,
}

impl BulletinBoardService {
    pub fn new(pool: SqlitePool) -> Self { Self { repo: BulletinBoardRepo(pool) } }

    pub async fn append(
        &self,
        auction_id: Uuid,
        kind: EntryKind,
        payload: serde_json::Value,
        signer: &ServerSigner,
    ) -> Result<BulletinBoardEntry, AuctionError> {
        let sequence = self.repo.next_sequence(auction_id).await?;
        let prev_hash_hex = match self.repo.get_head(auction_id).await? {
            Some(h) => h.entry_hash_hex,
            None => "0".repeat(64),
        };
        let prev_bytes = hex::decode(&prev_hash_hex)
            .unwrap_or_else(|_| vec![0u8; 32]);
        let prev_arr: [u8; 32] = prev_bytes.try_into().unwrap_or([0u8; 32]);

        let payload_json = serde_json::to_string(&payload)?;
        let payload_bytes = payload_json.as_bytes();

        let mut h = Sha256::new();
        h.update(b"auction-bb-entry-v1:");
        h.update(prev_arr);
        h.update((sequence as u64).to_le_bytes());
        h.update((payload_bytes.len() as u64).to_le_bytes());
        h.update(payload_bytes);
        let entry_hash: [u8; 32] = h.finalize().into();
        let entry_hash_hex = hex::encode(entry_hash);

        let sig = signer.sign(&entry_hash);
        let server_signature_hex = hex::encode(&sig);

        let entry = BulletinBoardEntry {
            sequence,
            auction_id,
            entry_kind: kind,
            payload_json,
            prev_hash_hex,
            entry_hash_hex,
            server_signature_hex,
            recorded_at: Utc::now(),
        };
        self.repo.insert(&entry).await?;
        Ok(entry)
    }

    pub async fn get_chain(&self, auction_id: Uuid) -> Result<Vec<BulletinBoardEntry>, AuctionError> {
        self.repo.find_by_auction(auction_id).await
    }

    pub async fn get_entry(
        &self, auction_id: Uuid, sequence: i64,
    ) -> Result<Option<BulletinBoardEntry>, AuctionError> {
        self.repo.find_by_sequence(auction_id, sequence).await
    }
}