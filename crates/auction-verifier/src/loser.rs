use auction_crypto::{PedersenCommitment, pedersen::PedersenGenerators, range_proof::{LoserRangeProof, RangeProofError}, schnorr::ProofOfOpening};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LoserVerifyError {
    #[error("Range proof invalid or Bid >= Winner: {0}")]
    InvalidProof(#[from] RangeProofError),
    #[error("Invalid stored commitment")]
    InvalidCommitment,
    #[error("Proof deserialize failed")]
    DeserializeError,
    #[error("Bidder claimed a higher value than the winner")]
    BidGreaterThanWinner,
}

pub fn verify_loser_proof(
    stored_commitment_hex: &str,
    proof_json: &str,
    winner_value: u64,
) -> Result<(), LoserVerifyError> {
    let bid_commitment = PedersenCommitment::from_hex(stored_commitment_hex)
        .map_err(|_| LoserVerifyError::InvalidCommitment)?;

    // Try to verify as a ProofOfOpening (Tie-breaker OR Challenge)
    if let Ok(poo) = serde_json::from_str::<ProofOfOpening>(proof_json) {
        let gens = PedersenGenerators::standard();
        if poo.verify(&gens).is_ok() {
            use subtle::ConstantTimeEq;
            if bool::from(poo.commitment.compress().as_bytes().ct_eq(&bid_commitment.to_bytes())) {
                // Cryptography is valid. Now check the protocol rules:
                if poo.revealed_value > winner_value {
                    return Err(LoserVerifyError::BidGreaterThanWinner);
                }
                return Ok(()); // Valid Tie-Breaker
            }
        }
    }

    // Fallback to normal zero-knowledge LoserRangeProof
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
