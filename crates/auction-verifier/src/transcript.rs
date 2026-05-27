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
    pub min_bid: u64,           
    pub max_bid: Option<u64>,   
    pub bid_step: u64,
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

    // Passiamo i parametri dell'asta al winner
    let winner_proof = match &t.winner {
        None => CheckResult::fail("no winner reveal"),
        Some(w) => match verify_winner_proof(&w.commitment_hex, w.revealed_value, &w.proof_json, gens, t.min_bid, t.max_bid, t.bid_step) {
            Ok(_) => CheckResult::ok(),
            Err(e) => CheckResult::fail(e),
        },
    };

    // Passiamo i parametri dell'asta ai loser
    // Passiamo i parametri dell'asta ai loser
    let loser_proofs = match &t.winner {
        None => CheckResult::fail("cannot verify losers without winner reveal"),
        Some(w) => {
            // CONTA QUANTI PARTECIPANTI CI SONO
            let total_bids = t.bulletin_board.iter()
                .filter(|e| matches!(e.entry_kind, auction_core::bulletin_board::EntryKind::SealedBid))
                .count();

            // SE MANCANO DELLE PROVE (es. B non ha mandato la loser proof)
            if total_bids > 0 && t.losers.len() != total_bids - 1 {
                 return VerificationResult {
                    auction_id: t.auction_id,
                    chain_integrity,
                    server_signatures,
                    winner_proof,
                    // FA FALLIRE LA VERIFICA SE MANCANO PROVE
                    loser_proofs: CheckResult::fail(format!("Missing loser proofs. Expected {}, found {}", total_bids - 1, t.losers.len())),
                    fully_valid: false,
                };
            }

            let losers: Vec<_> = t.losers.iter()
                .map(|l| (l.commitment_hex.clone(), l.proof_json.clone()))
                .collect();
            
            let errs = verify_all_loser_proofs(&losers, w.revealed_value);
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
mod e2e_tests {
    use super::*;
    use auction_core::bulletin_board::{BulletinBoardEntry, EntryKind};
    use auction_crypto::{
        pedersen::{BlindingFactor, PedersenCommitment, PedersenGenerators},
        schnorr::ProofOfOpening,
        range_proof::LoserRangeProof,
        signature::ServerSigner,
    };
    use chrono::Utc;
    use rand::rngs::OsRng;
    use uuid::Uuid;

    // Helper to generate a valid Bulletin Board entry
    fn mock_bb_entry(seq: i64, prev: [u8; 32], signer: &ServerSigner) -> (BulletinBoardEntry, [u8; 32]) {
        let payload = format!("mock-payload-{}", seq);
        let hash = BulletinBoardEntry::compute_hash(&prev, seq, payload.as_bytes());
        
        (BulletinBoardEntry {
            sequence: seq,
            auction_id: Uuid::new_v4(),
            entry_kind: EntryKind::SealedBid,
            payload_json: payload.clone(),
            prev_hash_hex: hex::encode(prev),
            entry_hash_hex: hex::encode(hash),
            server_signature_hex: hex::encode(signer.sign(&hash)),
            recorded_at: Utc::now(),
        }, hash)
    }

    #[test]
    fn test_full_auction_lifecycle_with_zk_proofs() {
        let mut rng = OsRng;
        let gens = PedersenGenerators::standard();
        let server_signer = ServerSigner::generate(&mut rng);

        // ==========================================
        // PHASE 1: AUCTION SETUP
        // ==========================================
        let auction_id = Uuid::new_v4();
        let min_bid = 100;
        let bid_step = 10;

        // ==========================================
        // PHASE 2: BIDDING (CLIENTS COMMIT)
        // ==========================================
        // Alice (Winner) bids 500
        let alice_id = Uuid::new_v4();
        let alice_val = 500;
        let alice_blind = BlindingFactor::random(&mut rng);
        let alice_commit = PedersenCommitment::commit(alice_val, &alice_blind, &gens);

        // Bob (Loser) bids 300
        let bob_id = Uuid::new_v4();
        let bob_val = 300;
        let bob_blind = BlindingFactor::random(&mut rng);
        let bob_commit = PedersenCommitment::commit(bob_val, &bob_blind, &gens);

        // Charlie (Loser) bids 450
        let charlie_id = Uuid::new_v4();
        let charlie_val = 450;
        let charlie_blind = BlindingFactor::random(&mut rng);
        let charlie_commit = PedersenCommitment::commit(charlie_val, &charlie_blind, &gens);

        // Simulate the Bulletin Board recording these bids
        let (e1, h1) = mock_bb_entry(1, [0u8; 32], &server_signer); // Alice Bid
        let (e2, h2) = mock_bb_entry(2, h1, &server_signer);        // Bob Bid
        let (e3, _)  = mock_bb_entry(3, h2, &server_signer);        // Charlie Bid

        // ==========================================
        // PHASE 3: WINNER REVEAL
        // ==========================================
        // Alice proves she bid 500 using a Schnorr Proof of Opening
        let alice_proof = ProofOfOpening::prove(alice_val, &alice_blind, &alice_commit, &gens, &mut rng);
        
        let winner_data = WinnerData {
            bidder_id: alice_id,
            bid_id: Uuid::new_v4(),
            commitment_hex: alice_commit.to_hex(),
            revealed_value: alice_val,
            proof_json: serde_json::to_string(&alice_proof).unwrap(),
        };

        // ==========================================
        // PHASE 4: LOSER PROOFS (ZERO-KNOWLEDGE)
        // ==========================================
        // Bob proves 300 < 500 using a Bulletproof
        let bob_zk_proof = LoserRangeProof::prove(bob_val, &bob_blind, alice_val).expect("Bob's proof failed");
        
        let bob_data = LoserData {
            bidder_id: bob_id,
            bid_id: Uuid::new_v4(),
            commitment_hex: bob_commit.to_hex(),
            proof_json: serde_json::to_string(&bob_zk_proof).unwrap(), // No revealed_value!
        };

        // Charlie proves 450 < 500 using a Bulletproof
        let charlie_zk_proof = LoserRangeProof::prove(charlie_val, &charlie_blind, alice_val).expect("Charlie's proof failed");
        
        let charlie_data = LoserData {
            bidder_id: charlie_id,
            bid_id: Uuid::new_v4(),
            commitment_hex: charlie_commit.to_hex(),
            proof_json: serde_json::to_string(&charlie_zk_proof).unwrap(), // No revealed_value!
        };

        // ==========================================
        // PHASE 5: INDEPENDENT AUDIT
        // ==========================================
        // Assemble the final transcript
        let transcript = AuctionTranscript {
            auction_id,
            min_bid,
            max_bid: None,
            bid_step,
            bulletin_board: vec![e1, e2, e3],
            winner: Some(winner_data),
            losers: vec![bob_data, charlie_data],
            server_verifier: server_signer.verifier(),
            pedersen_generators: gens,
        };

        // The verifier checks the entire auction outcome
        let verification_result = verify_auction_transcript(&transcript);

        // Ensure everything passed successfully
        assert!(verification_result.chain_integrity.passed, "Chain integrity failed");
        assert!(verification_result.server_signatures.passed, "Signatures failed");
        assert!(verification_result.winner_proof.passed, "Winner proof failed");
        assert!(verification_result.loser_proofs.passed, "Loser proofs failed: {:?}", verification_result.loser_proofs.error);
        assert!(verification_result.fully_valid, "The auction transcript is invalid!");
    }
}
