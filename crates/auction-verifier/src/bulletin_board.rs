use auction_core::bulletin_board::BulletinBoardEntry;
use auction_crypto::signature::ServerVerifier;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ChainVerifyError {
    #[error("entry {seq}: prev_hash does not link to previous entry_hash")]
    BrokenLink { seq: i64 },
    #[error("entry {seq}: recomputed hash does not match entry_hash")]
    HashMismatch { seq: i64 },
    #[error("entry {seq}: sequence gap (expected {expected}, got {got})")]
    SequenceGap { seq: i64, expected: i64, got: i64 },
    #[error("entry {seq}: server signature invalid")]
    InvalidSignature { seq: i64 },
    #[error("cannot decode hex field '{field}' on entry {seq}")]
    HexDecode { seq: i64, field: &'static str },
}

pub fn verify_chain_integrity(entries: &[BulletinBoardEntry]) -> Result<(), ChainVerifyError> {
    let mut prev_hash = [0u8; 32];
    for (idx, entry) in entries.iter().enumerate() {
        let expected_seq = idx as i64;
        if entry.sequence != expected_seq {
            return Err(ChainVerifyError::SequenceGap {
                seq: entry.sequence, expected: expected_seq, got: entry.sequence,
            });
        }
        let entry_prev = decode32(&entry.prev_hash_hex)
            .ok_or(ChainVerifyError::HexDecode { seq: entry.sequence, field: "prev_hash_hex" })?;
        if entry_prev != prev_hash {
            return Err(ChainVerifyError::BrokenLink { seq: entry.sequence });
        }
        let computed = compute_hash(&prev_hash, entry.sequence as u64, entry.payload_json.as_bytes());
        let stored = decode32(&entry.entry_hash_hex)
            .ok_or(ChainVerifyError::HexDecode { seq: entry.sequence, field: "entry_hash_hex" })?;
        if computed != stored {
            return Err(ChainVerifyError::HashMismatch { seq: entry.sequence });
        }
        prev_hash = computed;
    }
    Ok(())
}

pub fn verify_chain_with_signatures(
    entries: &[BulletinBoardEntry],
    verifier: &ServerVerifier,
) -> Result<(), ChainVerifyError> {
    verify_chain_integrity(entries)?;
    for entry in entries {
        let hash = decode32(&entry.entry_hash_hex)
            .ok_or(ChainVerifyError::HexDecode { seq: entry.sequence, field: "entry_hash_hex" })?;
        let sig = hex::decode(&entry.server_signature_hex)
            .map_err(|_| ChainVerifyError::HexDecode { seq: entry.sequence, field: "server_signature_hex" })?;
        verifier.verify(&hash, &sig)
            .map_err(|_| ChainVerifyError::InvalidSignature { seq: entry.sequence })?;
    }
    Ok(())
}

fn compute_hash(prev: &[u8; 32], seq: u64, payload: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"auction-bb-entry-v1:");
    h.update(prev);
    h.update(seq.to_le_bytes());
    h.update((payload.len() as u64).to_le_bytes());
    h.update(payload);
    h.finalize().into()
}

fn decode32(s: &str) -> Option<[u8; 32]> {
    hex::decode(s).ok()?.try_into().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use auction_core::bulletin_board::{BulletinBoardEntry, EntryKind};
    use auction_crypto::signature::ServerSigner;
    use chrono::Utc;
    use rand::rngs::OsRng;
    use uuid::Uuid;

    fn make_entry(seq: i64, prev: [u8; 32], payload: &str, signer: &ServerSigner) -> (BulletinBoardEntry, [u8; 32]) {
        let hash = compute_hash(&prev, seq as u64, payload.as_bytes());
        let sig = signer.sign(&hash);
        (BulletinBoardEntry {
            sequence: seq,
            auction_id: Uuid::new_v4(),
            entry_kind: EntryKind::AuctionCreate,
            payload_json: payload.to_string(),
            prev_hash_hex: hex::encode(prev),
            entry_hash_hex: hex::encode(hash),
            server_signature_hex: hex::encode(&sig),
            recorded_at: Utc::now(),
        }, hash)
    }

    fn build(n: usize) -> (Vec<BulletinBoardEntry>, ServerSigner) {
        let s = ServerSigner::generate(&mut OsRng);
        let mut entries = Vec::new();
        let mut prev = [0u8; 32];
        for i in 0..n {
            let (e, h) = make_entry(i as i64, prev, &format!("p{i}"), &s);
            prev = h; entries.push(e);
        }
        (entries, s)
    }

    #[test] fn empty_ok() { assert!(verify_chain_integrity(&[]).is_ok()); }
    #[test] fn valid_chain() { let (e,s)=build(5); assert!(verify_chain_with_signatures(&e,&s.verifier()).is_ok()); }
    #[test] fn tampered_payload() {
        let (mut e,_)=build(3); e[1].payload_json="evil".into();
        assert!(matches!(verify_chain_integrity(&e), Err(ChainVerifyError::HashMismatch{..})));
    }
    #[test] fn broken_link() {
        let (mut e,_)=build(3); e[2].prev_hash_hex="00".repeat(32);
        assert!(matches!(verify_chain_integrity(&e), Err(ChainVerifyError::BrokenLink{..})));
    }
    #[test] fn bad_sig() {
        let (e,_)=build(2); let w=ServerSigner::generate(&mut OsRng);
        assert!(verify_chain_with_signatures(&e,&w.verifier()).is_err());
    }
}