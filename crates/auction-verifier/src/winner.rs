use auction_crypto::{pedersen::PedersenGenerators, schnorr::ProofOfOpening};
use thiserror::Error;
use crate::commitment::{verify_proof_commitment_matches, CommitmentError};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WinnerVerifyError {
    #[error("Schnorr proof invalid: {0}")]
    InvalidProof(String),
    #[error("commitment mismatch: {0}")]
    CommitmentMismatch(#[from] CommitmentError),
    #[error("claimed value {request} differs from proof value {proof}")]
    ValueMismatch { request: u64, proof: u64 },
    #[error("proof JSON deserialize failed: {0}")]
    DeserializeError(String),
}

pub fn verify_winner_proof(
    stored_commitment_hex: &str,
    claimed_value: u64,
    proof_json: &str,
    gens: &PedersenGenerators,
) -> Result<(), WinnerVerifyError> {
    let proof: ProofOfOpening = serde_json::from_str(proof_json)
        .map_err(|e| WinnerVerifyError::DeserializeError(e.to_string()))?;
    if claimed_value != proof.revealed_value {
        return Err(WinnerVerifyError::ValueMismatch { request: claimed_value, proof: proof.revealed_value });
    }
    verify_proof_commitment_matches(stored_commitment_hex, &proof)?;
    proof.verify(gens).map_err(|e| WinnerVerifyError::InvalidProof(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use auction_crypto::pedersen::{BlindingFactor, PedersenCommitment, PedersenGenerators};
    use rand::rngs::OsRng;

    fn make(value: u64) -> (String, String, PedersenGenerators) {
        let g=PedersenGenerators::standard();
        let r=BlindingFactor::random(&mut OsRng);
        let c=PedersenCommitment::commit(value,&r,&g);
        let p=auction_crypto::schnorr::ProofOfOpening::prove(value,&r,&c,&g,&mut OsRng);
        (c.to_hex(), serde_json::to_string(&p).unwrap(), g)
    }

    #[test] fn valid_ok() { let (c,p,g)=make(10_000); assert!(verify_winner_proof(&c,10_000,&p,&g).is_ok()); }
    #[test] fn wrong_claimed_value() {
        let (c,p,g)=make(10_000);
        assert!(matches!(verify_winner_proof(&c,9_999,&p,&g), Err(WinnerVerifyError::ValueMismatch{..})));
    }
    #[test] fn wrong_commitment() {
        let (c1,_,g)=make(5_000); let (_,p2,_)=make(5_000);
        assert!(matches!(verify_winner_proof(&c1,5_000,&p2,&g), Err(WinnerVerifyError::CommitmentMismatch(_))));
    }
    #[test] fn bad_json() {
        let g=PedersenGenerators::standard();
        assert!(matches!(verify_winner_proof("aabb",100,"notjson",&g), Err(WinnerVerifyError::DeserializeError(_))));
    }
}