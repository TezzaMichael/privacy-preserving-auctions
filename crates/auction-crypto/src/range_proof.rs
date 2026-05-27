use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use curve25519_dalek_ng::{
    ristretto::CompressedRistretto as NgCompressed,
    scalar::Scalar as NgScalar,
};
use merlin::Transcript;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::pedersen::{BlindingFactor, PedersenCommitment};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RangeProofError {
    #[error("Failed to generate proof")]
    GenerationFailed,
    #[error("Failed to verify proof")]
    VerificationFailed,
    #[error("Bid is not strictly less than winner value")]
    Underflow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoserRangeProof {
    pub proof_bytes: Vec<u8>,
}

impl LoserRangeProof {
    pub fn prove(
        bid: u64,
        blinding: &BlindingFactor,
        winner_value: u64,
    ) -> Result<Self, RangeProofError> {
        if bid >= winner_value {
            return Err(RangeProofError::Underflow);
        }

        let delta = (winner_value - 1) - bid;

        // Bridge: Convert dalek Scalar to dalek-ng Scalar via bytes
        let delta_blinding_bytes = (-blinding.0).to_bytes();
        let delta_blinding_ng = NgScalar::from_bits(delta_blinding_bytes);

        let pc_gens = PedersenGens::default();
        let bp_gens = BulletproofGens::new(64, 1);
        let mut transcript = Transcript::new(b"auction-loser-proof");

        let (proof, _commitment) = RangeProof::prove_single(
            &bp_gens,
            &pc_gens,
            &mut transcript,
            delta,
            &delta_blinding_ng,
            64,
        )
        .map_err(|_| RangeProofError::GenerationFailed)?;

        Ok(Self {
            proof_bytes: proof.to_bytes(),
        })
    }

    pub fn verify(
        &self,
        bid_commitment: &PedersenCommitment,
        winner_value: u64,
    ) -> Result<(), RangeProofError> {
        let pc_gens = PedersenGens::default();
        let bp_gens = BulletproofGens::new(64, 1);
        let mut transcript = Transcript::new(b"auction-loser-proof");

        let s_minus_1_ng = NgScalar::from(winner_value - 1);
        let c_s_minus_1_ng = pc_gens.B * s_minus_1_ng;

        // Bridge: Convert dalek RistrettoPoint to dalek-ng RistrettoPoint
        let bid_point_bytes = bid_commitment.point.compress().to_bytes();
        let bid_point_ng = NgCompressed(bid_point_bytes)
            .decompress()
            .ok_or(RangeProofError::VerificationFailed)?;

        let c_delta_ng = c_s_minus_1_ng - bid_point_ng;

        let proof = RangeProof::from_bytes(&self.proof_bytes)
            .map_err(|_| RangeProofError::VerificationFailed)?;

        proof
            .verify_single(
                &bp_gens,
                &pc_gens,
                &mut transcript,
                &c_delta_ng.compress(),
                64,
            )
            .map_err(|_| RangeProofError::VerificationFailed)?;

        Ok(())
    }
}
