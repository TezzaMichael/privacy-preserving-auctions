use curve25519_dalek::{ristretto::RistrettoPoint, scalar::Scalar, traits::MultiscalarMul};
use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use thiserror::Error;

use crate::{
    fiat_shamir::FiatShamirTranscript,
    pedersen::{BlindingFactor, PedersenCommitment, PedersenGenerators, compressed_point_serde},
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProofError {
    #[error("Schnorr equation failed: s_v·G + s_r·H ≠ R + c·C")]
    InvalidSchnorrEquation,
    #[error("invalid point encoding in proof")]
    InvalidPoint,
    #[error("non-canonical scalar in proof response")]
    NonCanonicalScalar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScalarHex(pub Scalar);

impl Serialize for ScalarHex {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(self.0.to_bytes()))
    }
}
impl<'de> Deserialize<'de> for ScalarHex {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let bytes = hex::decode(String::deserialize(d)?).map_err(serde::de::Error::custom)?;
        Scalar::from_canonical_bytes(
            <[u8; 32]>::try_from(bytes).map_err(|_| serde::de::Error::custom("expected 32 bytes"))?,
        )
        .into_option()
        .map(ScalarHex)
        .ok_or_else(|| serde::de::Error::custom("non-canonical scalar"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfOpening {
    #[serde(with = "compressed_point_serde")]
    pub commitment: RistrettoPoint,
    #[serde(with = "compressed_point_serde")]
    pub nonce_commit: RistrettoPoint,
    pub s_value: ScalarHex,
    pub s_blinding: ScalarHex,
    pub revealed_value: u64,
}

impl ProofOfOpening {
    pub fn prove<R: RngCore + CryptoRng>(
        value: u64,
        blinding: &BlindingFactor,
        commitment: &PedersenCommitment,
        gens: &PedersenGenerators,
        rng: &mut R,
    ) -> Self {
        let (k_v, k_r) = (Scalar::random(rng), Scalar::random(rng));
        let nonce_commit = RistrettoPoint::multiscalar_mul([&k_v, &k_r], [&gens.g, &gens.h]);
        let c = build_transcript(gens, &commitment.point, &nonce_commit).challenge_scalar();
        Self {
            commitment: commitment.point,
            nonce_commit,
            s_value: ScalarHex(k_v + c * Scalar::from(value)),
            s_blinding: ScalarHex(k_r + c * blinding.0),
            revealed_value: value,
        }
    }

    pub fn verify(&self, gens: &PedersenGenerators) -> Result<(), ProofError> {
        let c = build_transcript(gens, &self.commitment, &self.nonce_commit).challenge_scalar();
        let lhs = RistrettoPoint::multiscalar_mul(
            [&self.s_value.0, &self.s_blinding.0],
            [&gens.g, &gens.h],
        );
        let rhs = self.nonce_commit + c * self.commitment;
        if bool::from(lhs.compress().as_bytes().ct_eq(rhs.compress().as_bytes())) {
            Ok(())
        } else {
            Err(ProofError::InvalidSchnorrEquation)
        }
    }

    pub fn value(&self) -> u64 { self.revealed_value }
    pub fn commitment_hex(&self) -> String { hex::encode(self.commitment.compress().to_bytes()) }
}

pub type LoserProof = ProofOfOpening;

fn build_transcript(
    gens: &PedersenGenerators,
    commitment: &RistrettoPoint,
    nonce_commit: &RistrettoPoint,
) -> FiatShamirTranscript {
    let mut t = FiatShamirTranscript::new();
    t.domain("auction-schnorr-proof-of-opening-v1")
        .absorb("G", gens.g.compress().as_bytes())
        .absorb("H", gens.h.compress().as_bytes())
        .absorb("commitment", commitment.compress().as_bytes())
        .absorb("nonce-commit", nonce_commit.compress().as_bytes());
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pedersen::{BlindingFactor, PedersenCommitment, PedersenGenerators};
    use rand::rngs::OsRng;

    fn prove(value: u64) -> (ProofOfOpening, PedersenGenerators) {
        let g = PedersenGenerators::standard();
        let r = BlindingFactor::random(&mut OsRng);
        let c = PedersenCommitment::commit(value, &r, &g);
        (ProofOfOpening::prove(value, &r, &c, &g, &mut OsRng), g)
    }

    #[test] fn roundtrip() { let (p,g) = prove(5_000); assert!(p.verify(&g).is_ok()); }
    #[test] fn zero_value() { let (p,g) = prove(0); assert!(p.verify(&g).is_ok()); }
    #[test] fn max_u64() { let (p,g) = prove(u64::MAX); assert!(p.verify(&g).is_ok()); }
    #[test] fn nonces_fresh() {
        let g = PedersenGenerators::standard();
        let r = BlindingFactor::random(&mut OsRng);
        let c = PedersenCommitment::commit(100, &r, &g);
        let p1 = ProofOfOpening::prove(100, &r, &c, &g, &mut OsRng);
        let p2 = ProofOfOpening::prove(100, &r, &c, &g, &mut OsRng);
        assert_ne!(p1.nonce_commit.compress().as_bytes(), p2.nonce_commit.compress().as_bytes());
    }
    #[test] fn tampered_s_value_fails() {
        let (mut p, g) = prove(1_000);
        p.s_value = ScalarHex(Scalar::random(&mut OsRng));
        assert_eq!(p.verify(&g), Err(ProofError::InvalidSchnorrEquation));
    }
    #[test] fn tampered_s_blinding_fails() {
        let (mut p, g) = prove(2_000);
        p.s_blinding = ScalarHex(Scalar::random(&mut OsRng));
        assert_eq!(p.verify(&g), Err(ProofError::InvalidSchnorrEquation));
    }
    #[test] fn tampered_nonce_fails() {
        let (mut p, g) = prove(3_000);
        let (p2, _) = prove(3_000);
        p.nonce_commit = p2.nonce_commit;
        assert_eq!(p.verify(&g), Err(ProofError::InvalidSchnorrEquation));
    }
    #[test] fn tampered_commitment_fails() {
        let g = PedersenGenerators::standard();
        let r1 = BlindingFactor::random(&mut OsRng);
        let r2 = BlindingFactor::random(&mut OsRng);
        let c1 = PedersenCommitment::commit(100, &r1, &g);
        let c2 = PedersenCommitment::commit(200, &r2, &g);
        let mut p = ProofOfOpening::prove(100, &r1, &c1, &g, &mut OsRng);
        p.commitment = c2.point;
        assert_eq!(p.verify(&g), Err(ProofError::InvalidSchnorrEquation));
    }
    #[test] fn wrong_generators_fails() {
        let g1 = PedersenGenerators::standard();
        let g2 = PedersenGenerators { g: g1.g, h: PedersenGenerators::hash_to_point(b"other") };
        let r = BlindingFactor::random(&mut OsRng);
        let c = PedersenCommitment::commit(500, &r, &g1);
        let p = ProofOfOpening::prove(500, &r, &c, &g1, &mut OsRng);
        assert_eq!(p.verify(&g2), Err(ProofError::InvalidSchnorrEquation));
    }
    #[test] fn json_roundtrip() {
        let (p, g) = prove(7_777);
        let p2: ProofOfOpening = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert!(p2.verify(&g).is_ok());
        assert_eq!(p2.revealed_value, 7_777);
    }
    #[test] fn scalar_hex_serde() {
        let s = ScalarHex(Scalar::random(&mut OsRng));
        let s2: ScalarHex = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(s.0.to_bytes(), s2.0.to_bytes());
    }
}