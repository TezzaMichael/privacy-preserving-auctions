use auction_crypto::{PedersenCommitment, pedersen::PedersenGenerators, range_proof::{LoserRangeProof, RangeProofError}, schnorr::ProofOfOpening};
use thiserror::Error;
use crate::commitment::{verify_proof_commitment_matches, CommitmentError};


#[derive(Debug, Error, PartialEq, Eq)]
pub enum LoserVerifyError {
    // #[error("Schnorr proof invalid: {0}")]
    // InvalidProof(String),
    // #[error("commitment mismatch: {0}")]
    // CommitmentMismatch(#[from] CommitmentError),
    // #[error("loser value ({loser}) >= winner value ({winner})")]
    // NotALoser { loser: u64, winner: u64 },
    // #[error("claimed value {request} differs from proof value {proof}")]
    // ValueMismatch { request: u64, proof: u64 },
    // #[error("proof JSON deserialize failed: {0}")]
    // DeserializeError(String),
    // #[error("L'offerta rivelata è inferiore al minimo consentito")]
    // BelowMinimum,
    // #[error("L'offerta rivelata è superiore al massimo consentito")]
    // AboveMaximum,
    // #[error("L'offerta rivelata non rispetta i salti (step) stabiliti")]
    // InvalidStep,
    #[error("Range proof invalid or Bid >= Winner: {0}")]
    InvalidProof(#[from] RangeProofError),
    #[error("Invalid stored commitment")]
    InvalidCommitment,
    #[error("Proof deserialize failed")]
    DeserializeError,
}

pub fn verify_loser_proof(
    stored_commitment_hex: &str,
    proof_json: &str,
    winner_value: u64,
) -> Result<(), LoserVerifyError> {
    let bid_commitment = PedersenCommitment::from_hex(stored_commitment_hex)
        .map_err(|_| LoserVerifyError::InvalidCommitment)?;

    let proof: LoserRangeProof = serde_json::from_str(proof_json)
        .map_err(|_| LoserVerifyError::DeserializeError)?;

    proof.verify(&bid_commitment, winner_value)?;

    Ok(())
}

pub fn verify_all_loser_proofs(
    losers: &[(String, String)], 
    winner_value: u64,
) -> Vec<(usize, LoserVerifyError)> {
    losers.iter().enumerate()
        .filter_map(|(i, (c, p))| {
            verify_loser_proof(c, p, winner_value).err().map(|e| (i, e))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use auction_crypto::pedersen::{BlindingFactor, PedersenCommitment, PedersenGenerators};
    use auction_crypto::range_proof::LoserRangeProof;
    use rand::rngs::OsRng;

    fn make_proof(bid: u64, winner_value: u64) -> (String, String) {
        let g = PedersenGenerators::standard();
        let r = BlindingFactor::random(&mut OsRng);
        let c = PedersenCommitment::commit(bid, &r, &g);
        
        // If bid >= winner_value, this will return an error, so we unwrap for valid tests
        let p = LoserRangeProof::prove(bid, &r, winner_value).unwrap_or(LoserRangeProof { proof_bytes: vec![] });
        (c.to_hex(), serde_json::to_string(&p).unwrap())
    }

    #[test] 
    fn valid_ok() { 
        // Bid 400, Winner 500
        let (c, p) = make_proof(400, 500); 
        assert!(verify_loser_proof(&c, &p, 500).is_ok()); 
    }
    
    #[test] 
    fn equal_winner_fails_at_generation() {
        let g = PedersenGenerators::standard();
        let r = BlindingFactor::random(&mut OsRng);
        // Trying to generate a proof for 500 when the winner is 500 should fail immediately
        assert!(LoserRangeProof::prove(500, &r, 500).is_err());
    }
    
    #[test] 
    fn verify_all_empty() {
        assert!(verify_all_loser_proofs(&[], 1000).is_empty());
    }
    
    #[test] 
    fn verify_all_one_fail() {
        let (c1, p1) = make_proof(100, 500);
        // Simulate a broken/invalid proof by submitting empty bytes
        let (c2, p2) = (
            PedersenCommitment::commit(900, &BlindingFactor::random(&mut OsRng), &PedersenGenerators::standard()).to_hex(),
            serde_json::to_string(&LoserRangeProof { proof_bytes: vec![] }).unwrap()
        );
        
        let errs = verify_all_loser_proofs(&[(c1, p1), (c2, p2)], 500);
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].0, 1); // The second proof (index 1) fails
    }
}
