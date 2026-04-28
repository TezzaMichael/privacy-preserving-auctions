use serde::{Deserialize, Serialize};
use uuid::Uuid;
use auction_core::bulletin_board::BulletinBoardEntry;
use auction_crypto::{pedersen::PedersenGenerators, signature::ServerVerifier};
use crate::{
    bulletin_board::{verify_chain_integrity, verify_chain_with_signatures},
    loser::verify_all_loser_proofs,
    winner::verify_winner_proof,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionTranscript {
    pub auction_id: Uuid,
    pub bulletin_board: Vec<BulletinBoardEntry>,
    pub winner: Option<WinnerData>,
    pub losers: Vec<LoserData>,
    pub server_verifier: ServerVerifier,
    pub pedersen_generators: PedersenGenerators,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinnerData {
    pub bidder_id: Uuid,
    pub bid_id: Uuid,
    pub commitment_hex: String,
    pub revealed_value: u64,
    pub proof_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoserData {
    pub bidder_id: Uuid,
    pub bid_id: Uuid,
    pub commitment_hex: String,
    pub revealed_value: u64,
    pub proof_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub auction_id: Uuid,
    pub chain_integrity: CheckResult,
    pub server_signatures: CheckResult,
    pub winner_proof: CheckResult,
    pub loser_proofs: CheckResult,
    pub fully_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub passed: bool,
    pub error: Option<String>,
}

impl CheckResult {
    fn ok() -> Self { Self { passed: true, error: None } }
    fn fail(e: impl ToString) -> Self { Self { passed: false, error: Some(e.to_string()) } }
}

pub fn verify_auction_transcript(t: &AuctionTranscript) -> VerificationResult {
    let gens = &t.pedersen_generators;

    let chain_integrity = match verify_chain_integrity(&t.bulletin_board) {
        Ok(_) => CheckResult::ok(),
        Err(e) => CheckResult::fail(e),
    };

    let server_signatures = match verify_chain_with_signatures(&t.bulletin_board, &t.server_verifier) {
        Ok(_) => CheckResult::ok(),
        Err(e) => CheckResult::fail(e),
    };

    let winner_proof = match &t.winner {
        None => CheckResult::fail("no winner reveal"),
        Some(w) => match verify_winner_proof(&w.commitment_hex, w.revealed_value, &w.proof_json, gens) {
            Ok(_) => CheckResult::ok(),
            Err(e) => CheckResult::fail(e),
        },
    };

    let loser_proofs = match &t.winner {
        None => CheckResult::fail("cannot verify losers without winner reveal"),
        Some(w) => {
            let losers: Vec<_> = t.losers.iter()
                .map(|l| (l.commitment_hex.clone(), l.revealed_value, l.proof_json.clone()))
                .collect();
            let errs = verify_all_loser_proofs(&losers, w.revealed_value, gens);
            if errs.is_empty() {
                CheckResult::ok()
            } else {
                CheckResult::fail(errs.iter().map(|(i,e)| format!("loser[{i}]: {e}")).collect::<Vec<_>>().join("; "))
            }
        }
    };

    let fully_valid = chain_integrity.passed && server_signatures.passed
        && winner_proof.passed && loser_proofs.passed;

    VerificationResult {
        auction_id: t.auction_id,
        chain_integrity,
        server_signatures,
        winner_proof,
        loser_proofs,
        fully_valid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auction_core::bulletin_board::{BulletinBoardEntry, EntryKind};
    use auction_crypto::{
        pedersen::{BlindingFactor, PedersenCommitment},
        schnorr::ProofOfOpening,
        signature::ServerSigner,
    };
    use chrono::Utc;
    use rand::rngs::OsRng;
    use sha2::{Digest, Sha256};

    fn bb_entry(seq: i64, prev: [u8; 32], payload: &str, signer: &ServerSigner) -> (BulletinBoardEntry, [u8; 32]) {
        let pb = payload.as_bytes();
        let mut h = Sha256::new();
        h.update(b"auction-bb-entry-v1:");
        h.update(prev);
        h.update((seq as u64).to_le_bytes());
        h.update((pb.len() as u64).to_le_bytes());
        h.update(pb);
        let hash: [u8; 32] = h.finalize().into();
        (BulletinBoardEntry {
            sequence: seq,
            auction_id: Uuid::new_v4(),
            entry_kind: EntryKind::SealedBid,
            payload_json: payload.to_string(),
            prev_hash_hex: hex::encode(prev),
            entry_hash_hex: hex::encode(hash),
            server_signature_hex: hex::encode(signer.sign(&hash)),
            recorded_at: Utc::now(),
        }, hash)
    }

    fn valid_transcript() -> AuctionTranscript {
        let signer = ServerSigner::generate(&mut OsRng);
        let gens = PedersenGenerators::standard();

        let rw = BlindingFactor::random(&mut OsRng);
        let cw = PedersenCommitment::commit(1000, &rw, &gens);
        let pw = ProofOfOpening::prove(1000, &rw, &cw, &gens, &mut OsRng);

        let rl = BlindingFactor::random(&mut OsRng);
        let cl = PedersenCommitment::commit(500, &rl, &gens);
        let pl = ProofOfOpening::prove(500, &rl, &cl, &gens, &mut OsRng);

        let (e0, h0) = bb_entry(0, [0u8;32], "create", &signer);
        let (e1, _)  = bb_entry(1, h0, "bids", &signer);

        AuctionTranscript {
            auction_id: Uuid::new_v4(),
            bulletin_board: vec![e0, e1],
            winner: Some(WinnerData {
                bidder_id: Uuid::new_v4(), bid_id: Uuid::new_v4(),
                commitment_hex: cw.to_hex(), revealed_value: 1000,
                proof_json: serde_json::to_string(&pw).unwrap(),
            }),
            losers: vec![LoserData {
                bidder_id: Uuid::new_v4(), bid_id: Uuid::new_v4(),
                commitment_hex: cl.to_hex(), revealed_value: 500,
                proof_json: serde_json::to_string(&pl).unwrap(),
            }],
            server_verifier: signer.verifier(),
            pedersen_generators: gens,
        }
    }

    #[test] fn fully_valid() {
        assert!(verify_auction_transcript(&valid_transcript()).fully_valid);
    }
    #[test] fn tampered_bb_fails_chain() {
        let mut t = valid_transcript();
        t.bulletin_board[0].payload_json = "evil".into();
        let r = verify_auction_transcript(&t);
        assert!(!r.chain_integrity.passed && !r.fully_valid);
    }
    #[test] fn no_winner_fails() {
        let mut t = valid_transcript();
        t.winner = None;
        assert!(!verify_auction_transcript(&t).winner_proof.passed);
    }
    #[test] fn loser_higher_than_winner_fails() {
        let mut t = valid_transcript();
        let gens = t.pedersen_generators.clone();
        let r = BlindingFactor::random(&mut OsRng);
        let c = PedersenCommitment::commit(2000, &r, &gens);
        let p = ProofOfOpening::prove(2000, &r, &c, &gens, &mut OsRng);
        t.losers[0] = LoserData {
            bidder_id: Uuid::new_v4(), bid_id: Uuid::new_v4(),
            commitment_hex: c.to_hex(), revealed_value: 2000,
            proof_json: serde_json::to_string(&p).unwrap(),
        };
        assert!(!verify_auction_transcript(&t).loser_proofs.passed);
    }
}