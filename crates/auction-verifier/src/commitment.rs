use auction_crypto::{pedersen::{PedersenCommitment, PedersenGenerators}, schnorr::ProofOfOpening};
use subtle::ConstantTimeEq;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CommitmentError {
    #[error("invalid commitment hex: {0}")]
    InvalidHex(String),
    #[error("proof commitment does not match stored commitment")]
    CommitmentMismatch,
}

pub fn verify_proof_commitment_matches(
    stored_hex: &str,
    proof: &ProofOfOpening,
) -> Result<(), CommitmentError> {
    let stored = PedersenCommitment::from_hex(stored_hex)
        .map_err(|e| CommitmentError::InvalidHex(e.to_string()))?;
    if !bool::from(proof.commitment.compress().as_bytes().ct_eq(&stored.to_bytes())) {
        return Err(CommitmentError::CommitmentMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use auction_crypto::pedersen::{BlindingFactor, PedersenCommitment, PedersenGenerators};
    use rand::rngs::OsRng;

    #[test] fn matching_ok() {
        let g=PedersenGenerators::standard();
        let r=BlindingFactor::random(&mut OsRng);
        let c=PedersenCommitment::commit(100,&r,&g);
        let p=auction_crypto::schnorr::ProofOfOpening::prove(100,&r,&c,&g,&mut OsRng);
        assert!(verify_proof_commitment_matches(&c.to_hex(),&p).is_ok());
    }
    #[test] fn mismatch_detected() {
        let g=PedersenGenerators::standard();
        let r1=BlindingFactor::random(&mut OsRng);
        let r2=BlindingFactor::random(&mut OsRng);
        let c1=PedersenCommitment::commit(100,&r1,&g);
        let c2=PedersenCommitment::commit(200,&r2,&g);
        let p=auction_crypto::schnorr::ProofOfOpening::prove(100,&r1,&c1,&g,&mut OsRng);
        assert_eq!(verify_proof_commitment_matches(&c2.to_hex(),&p), Err(CommitmentError::CommitmentMismatch));
    }
}