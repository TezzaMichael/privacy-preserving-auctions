use chrono::Utc;
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::models::{
    app_state::AppState,
    enums::EntryKind,
    bulletin_board::BulletinEntry,
};


/// Appende una nuova entry al Bulletin Board di un'asta
pub fn append_entry(
    state: &AppState,
    auction_id: Uuid,
    kind: EntryKind,
    payload: Value,
) -> BulletinEntry {

    let mut bb_map = state.bb.write().unwrap();

    let entries = bb_map.entry(auction_id).or_insert_with(Vec::new);

    let sequence = (entries.len() as u64) + 1;

    let prev_hash = entries
        .last()
        .map(|e| e.entry_hash.clone())
        .unwrap_or_else(BulletinEntry::genesis_hash);

    let timestamp = Utc::now();

    let entry_hash = compute_entry_hash(
        &prev_hash,
        sequence,
        &kind,
        auction_id,
        &timestamp,
        &payload,
    );

    let server_sig = sign_entry_hash(
        &entry_hash,
        &state.server_private_key,
    );

    let entry = BulletinEntry {

        id: Uuid::new_v4(),

        sequence,

        kind,

        auction_id,

        timestamp,

        payload,

        prev_hash,

        entry_hash,

        server_sig,
    };

    entries.push(entry.clone());

    entry
}



/// Calcola hash entry secondo schema:
///
/// SHA256(prev_hash | sequence | kind | auction_id | timestamp | payload)
///
pub fn compute_entry_hash(
    prev_hash: &str,
    sequence: u64,
    kind: &EntryKind,
    auction_id: Uuid,
    timestamp: &chrono::DateTime<Utc>,
    payload: &Value,
) -> String {

    let mut hasher = Sha256::new();

    hasher.update(prev_hash);

    hasher.update(sequence.to_be_bytes());

    hasher.update(format!("{:?}", kind));

    hasher.update(auction_id.to_string());

    hasher.update(timestamp.timestamp().to_be_bytes());

    hasher.update(payload.to_string());

    let result = hasher.finalize();

    hex::encode(result)
}



/// Firma hash entry con chiave server
///
/// ATTENZIONE:
/// versione placeholder (firma simulata)
/// sostituire con Ed25519 reale in crypto/signature.rs
///
fn sign_entry_hash(
    entry_hash: &str,
    server_private_key: &str,
) -> String {

    let mut hasher = Sha256::new();

    hasher.update(entry_hash);

    hasher.update(server_private_key);

    hex::encode(hasher.finalize())
}



/// Verifica integrità hash-chain Bulletin Board
///
/// Controlla:
///
/// prev_hash[n] == entry_hash[n-1]
///
/// entry_hash corretto
///
pub fn verify_chain(
    entries: &[BulletinEntry],
) -> bool {

    if entries.is_empty() {
        return true;
    }

    let mut previous_hash =
        BulletinEntry::genesis_hash();

    for entry in entries {

        let recomputed_hash =
            compute_entry_hash(
                &entry.prev_hash,
                entry.sequence,
                &entry.kind,
                entry.auction_id,
                &entry.timestamp,
                &entry.payload,
            );

        if entry.prev_hash != previous_hash {
            return false;
        }

        if entry.entry_hash != recomputed_hash {
            return false;
        }

        previous_hash =
            entry.entry_hash.clone();
    }

    true
}



/// Recupera tutte le entry di una specifica asta
pub fn get_entries_for_auction(
    state: &AppState,
    auction_id: Uuid,
) -> Vec<BulletinEntry> {

    let bb_map = state.bb.read().unwrap();

    bb_map
        .get(&auction_id)
        .cloned()
        .unwrap_or_default()
}



/// Restituisce ultima entry hash (utile per audit incrementale)
pub fn get_last_hash(
    state: &AppState,
    auction_id: Uuid,
) -> Option<String> {

    let bb_map = state.bb.read().unwrap();

    bb_map
        .get(&auction_id)
        .and_then(|entries| entries.last())
        .map(|entry| entry.entry_hash.clone())
}