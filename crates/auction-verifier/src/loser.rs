use auction_crypto::{pedersen::PedersenGenerators, schnorr::ProofOfOpening};
use thiserror::Error;
use crate::commitment::{verify_proof_commitment_matches, CommitmentError};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LoserVerifyError {
    #[error("Schnorr proof invalid: {0}")]
    InvalidProof(String),
    #[error("commitment mismatch: {0}")]
    CommitmentMismatch(#[from] CommitmentError),
    #[error("loser value ({loser}) >= winner value ({winner})")]
    NotALoser { loser: u64, winner: u64 },
    #[error("claimed value {request} differs from proof value {proof}")]
    ValueMismatch { request: u64, proof: u64 },
    #[error("proof JSON deserialize failed: {0}")]
    DeserializeError(String),
}

pub fn verify_loser_proof(
    stored_commitment_hex: &str,
    claimed_value: u64,
    proof_json: &str,
    winner_value: u64,
    gens: &PedersenGenerators,
) -> Result<(), LoserVerifyError> {
    let proof: ProofOfOpening = serde_json::from_str(proof_json)
        .map_err(|e| LoserVerifyError::DeserializeError(e.to_string()))?;
    if claimed_value != proof.revealed_value {
        return Err(LoserVerifyError::ValueMismatch { request: claimed_value, proof: proof.revealed_value });
    }
    if proof.revealed_value >= winner_value {
        return Err(LoserVerifyError::NotALoser { loser: proof.revealed_value, winner: winner_value });
    }
    verify_proof_commitment_matches(stored_commitment_hex, &proof)?;
    proof.verify(gens).map_err(|e| LoserVerifyError::InvalidProof(e.to_string()))
}

pub fn verify_all_loser_proofs(
    losers: &[(String, u64, String)],
    winner_value: u64,
    gens: &PedersenGenerators,
) -> Vec<(usize, LoserVerifyError)> {
    losers.iter().enumerate()
        .filter_map(|(i, (c, v, p))| {
            verify_loser_proof(c, *v, p, winner_value, gens).err().map(|e| (i, e))
        })
        .collect()
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

    #[test] fn valid_ok() { let (c,p,g)=make(400); assert!(verify_loser_proof(&c,400,&p,500,&g).is_ok()); }
    #[test] fn equal_winner_fails() {
        let (c,p,g)=make(500);
        assert!(matches!(verify_loser_proof(&c,500,&p,500,&g), Err(LoserVerifyError::NotALoser{..})));
    }
    #[test] fn higher_than_winner_fails() {
        let (c,p,g)=make(600);
        assert!(matches!(verify_loser_proof(&c,600,&p,500,&g), Err(LoserVerifyError::NotALoser{..})));
    }
    #[test] fn value_mismatch_fails() {
        let (c,p,g)=make(400);
        assert!(matches!(verify_loser_proof(&c,300,&p,500,&g), Err(LoserVerifyError::ValueMismatch{..})));
    }
    #[test] fn verify_all_empty() {
        let g=PedersenGenerators::standard();
        assert!(verify_all_loser_proofs(&[],1000,&g).is_empty());
    }
    #[test] fn verify_all_one_fail() {
        let g=PedersenGenerators::standard();
        let (c1,p1,_)=make(100);
        let (c2,p2,_)=make(900);
        let errs=verify_all_loser_proofs(&[(c1,100,p1),(c2,900,p2)],500,&g);
        assert_eq!(errs.len(),1);
        assert_eq!(errs[0].0,1);
    }
}