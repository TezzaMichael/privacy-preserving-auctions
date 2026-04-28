use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ChainError {
    #[error("sequence mismatch: expected {expected}, got {got}")]
    SequenceMismatch { expected: u64, got: u64 },
    #[error("prev_hash does not match previous entry_hash")]
    PrevHashMismatch,
    #[error("entry_hash does not match recomputed hash")]
    EntryHashMismatch,
    #[error("server signature invalid on entry {sequence}")]
    InvalidSignature { sequence: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HexBytes32(pub [u8; 32]);

impl HexBytes32 {
    pub const ZERO: Self = Self([0u8; 32]);
}

impl Serialize for HexBytes32 {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(self.0))
    }
}
impl<'de> Deserialize<'de> for HexBytes32 {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        hex::decode(String::deserialize(d)?)
            .map_err(serde::de::Error::custom)?
            .try_into()
            .map(Self)
            .map_err(|_| serde::de::Error::custom("expected 32 bytes"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainEntry {
    pub sequence: u64,
    pub prev_hash: HexBytes32,
    pub entry_hash: HexBytes32,
    #[serde(with = "bytes_as_hex")]
    pub server_signature: Vec<u8>,
    #[serde(with = "bytes_as_hex")]
    pub payload: Vec<u8>,
}

#[derive(Default)]
pub struct HashChain {
    entries: Vec<ChainEntry>,
}

impl HashChain {
    pub fn new() -> Self { Self::default() }

    pub fn restore(entries: Vec<ChainEntry>) -> Result<Self, ChainError> {
        verify_chain(&entries)?;
        Ok(Self { entries })
    }

    pub fn head_hash(&self) -> [u8; 32] {
        self.entries.last().map(|e| e.entry_hash.0).unwrap_or([0u8; 32])
    }

    pub fn next_sequence(&self) -> u64 { self.entries.len() as u64 }

    pub fn compute_entry_hash(prev: &[u8; 32], seq: u64, payload: &[u8]) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(b"auction-bb-entry-v1:");
        h.update(prev);
        h.update(seq.to_le_bytes());
        h.update((payload.len() as u64).to_le_bytes());
        h.update(payload);
        h.finalize().into()
    }

    pub fn build_entry(&self, payload: Vec<u8>) -> ChainEntry {
        let seq = self.next_sequence();
        let prev = HexBytes32(self.head_hash());
        ChainEntry {
            sequence: seq,
            prev_hash: prev,
            entry_hash: HexBytes32(Self::compute_entry_hash(&prev.0, seq, &payload)),
            server_signature: vec![],
            payload,
        }
    }

    pub fn append(&mut self, entry: ChainEntry) -> Result<(), ChainError> {
        let exp_seq = self.next_sequence();
        if entry.sequence != exp_seq {
            return Err(ChainError::SequenceMismatch { expected: exp_seq, got: entry.sequence });
        }
        if entry.prev_hash.0 != self.head_hash() {
            return Err(ChainError::PrevHashMismatch);
        }
        let expected = Self::compute_entry_hash(&entry.prev_hash.0, entry.sequence, &entry.payload);
        if entry.entry_hash.0 != expected {
            return Err(ChainError::EntryHashMismatch);
        }
        self.entries.push(entry);
        Ok(())
    }

    pub fn entries(&self) -> &[ChainEntry] { &self.entries }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn get(&self, seq: u64) -> Option<&ChainEntry> { self.entries.get(seq as usize) }
}

pub fn verify_chain(entries: &[ChainEntry]) -> Result<(), ChainError> {
    let mut prev = [0u8; 32];
    for (i, e) in entries.iter().enumerate() {
        let exp = i as u64;
        if e.sequence != exp { return Err(ChainError::SequenceMismatch { expected: exp, got: e.sequence }); }
        if e.prev_hash.0 != prev { return Err(ChainError::PrevHashMismatch); }
        let computed = HashChain::compute_entry_hash(&prev, e.sequence, &e.payload);
        if e.entry_hash.0 != computed { return Err(ChainError::EntryHashMismatch); }
        prev = computed;
    }
    Ok(())
}

pub fn verify_chain_with_sigs(
    entries: &[ChainEntry],
    verifier: &crate::signature::ServerVerifier,
) -> Result<(), ChainError> {
    verify_chain(entries)?;
    for e in entries {
        verifier.verify(&e.entry_hash.0, &e.server_signature)
            .map_err(|_| ChainError::InvalidSignature { sequence: e.sequence })?;
    }
    Ok(())
}

mod bytes_as_hex {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(b: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(b))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        hex::decode(String::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::ServerSigner;
    use rand::rngs::OsRng;

    fn signed_entry(chain: &HashChain, payload: &[u8], signer: &ServerSigner) -> ChainEntry {
        let mut e = chain.build_entry(payload.to_vec());
        e.server_signature = signer.sign(&e.entry_hash.0);
        e
    }

    fn build(n: usize) -> (HashChain, ServerSigner) {
        let s = ServerSigner::generate(&mut OsRng);
        let mut c = HashChain::new();
        for i in 0..n { c.append(signed_entry(&c, format!("ev-{i}").as_bytes(), &s)).unwrap(); }
        (c, s)
    }

    #[test] fn empty_ok() { assert!(verify_chain(&[]).is_ok()); }
    #[test] fn single_ok() { let (c,s) = build(1); assert!(verify_chain_with_sigs(c.entries(), &s.verifier()).is_ok()); }
    #[test] fn multi_ok() { let (c,s) = build(5); assert!(verify_chain_with_sigs(c.entries(), &s.verifier()).is_ok()); }
    #[test] fn genesis_zeros() {
        let s = ServerSigner::generate(&mut OsRng);
        let c = HashChain::new();
        assert_eq!(signed_entry(&c, b"x", &s).prev_hash.0, [0u8; 32]);
    }
    #[test] fn links_correct() {
        let (c, _) = build(4);
        for i in 1..c.len() {
            assert_eq!(c.entries()[i].prev_hash.0, c.entries()[i-1].entry_hash.0);
        }
    }
    #[test] fn tampered_payload() {
        let (c,_) = build(3);
        let mut e = c.entries().to_vec();
        e[1].payload = b"evil".to_vec();
        assert!(matches!(verify_chain(&e), Err(ChainError::EntryHashMismatch)));
    }
    #[test] fn wrong_seq() {
        let (c,s) = build(1);
        let mut bad = c.build_entry(b"x".to_vec());
        bad.sequence = 99;
        bad.server_signature = s.sign(&bad.entry_hash.0);
        assert!(matches!(c.clone_for_test().append(bad), Err(ChainError::SequenceMismatch { .. })));
    }
    #[test] fn bad_sig() {
        let (c,_) = build(2);
        let wrong = ServerSigner::generate(&mut OsRng);
        assert!(verify_chain_with_sigs(c.entries(), &wrong.verifier()).is_err());
    }
    #[test] fn restore_valid() {
        let (c,_) = build(4);
        let r = HashChain::restore(c.entries().to_vec()).unwrap();
        assert_eq!(r.head_hash(), c.head_hash());
    }
    #[test] fn restore_tampered_fails() {
        let (c,_) = build(3);
        let mut e = c.entries().to_vec();
        e[0].payload = b"evil".to_vec();
        assert!(HashChain::restore(e).is_err());
    }
    #[test] fn get_by_seq() {
        let (c,_) = build(5);
        for i in 0..5u64 { assert_eq!(c.get(i).unwrap().sequence, i); }
        assert!(c.get(99).is_none());
    }
    #[test] fn entry_json_roundtrip() {
        let (c,_) = build(1);
        let e = &c.entries()[0];
        let e2: ChainEntry = serde_json::from_str(&serde_json::to_string(e).unwrap()).unwrap();
        assert_eq!(e.entry_hash, e2.entry_hash);
    }
}

impl HashChain {
    fn clone_for_test(&self) -> HashChain {
        HashChain::restore(self.entries().to_vec()).unwrap()
    }
}