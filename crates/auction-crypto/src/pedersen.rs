use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_POINT,
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
    traits::MultiscalarMul,
};
use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_512};
use subtle::ConstantTimeEq;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PedersenError {
    #[error("invalid compressed Ristretto255 point")]
    InvalidPoint,
    #[error("commitment verification failed")]
    VerificationFailed,
    #[error("invalid hex: {0}")]
    HexDecode(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PedersenGenerators {
    #[serde(with = "compressed_point_serde")]
    pub g: RistrettoPoint,
    #[serde(with = "compressed_point_serde")]
    pub h: RistrettoPoint,
}

impl PedersenGenerators {
    pub fn standard() -> Self {
        Self {
            g: RISTRETTO_BASEPOINT_POINT,
            h: Self::hash_to_point(b"auction-pedersen-H-generator-v1"),
        }
    }

    pub fn hash_to_point(domain: &[u8]) -> RistrettoPoint {
        let mut hasher = Sha3_512::new();
        hasher.update(b"auction-hash-to-point-v1:");
        hasher.update((domain.len() as u64).to_le_bytes());
        hasher.update(domain);
        RistrettoPoint::from_uniform_bytes(&hasher.finalize().into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PedersenCommitment {
    #[serde(with = "compressed_point_serde")]
    pub point: RistrettoPoint,
}

impl PedersenCommitment {
    pub fn commit(value: u64, blinding: &BlindingFactor, gens: &PedersenGenerators) -> Self {
        Self {
            point: RistrettoPoint::multiscalar_mul(
                [&Scalar::from(value), &blinding.0],
                [&gens.g, &gens.h],
            ),
        }
    }

    pub fn verify(&self, value: u64, blinding: &BlindingFactor, gens: &PedersenGenerators) -> bool {
        bool::from(
            self.point.compress().as_bytes().ct_eq(
                Self::commit(value, blinding, gens).point.compress().as_bytes(),
            ),
        )
    }

    pub fn to_bytes(&self) -> [u8; 32] { self.point.compress().to_bytes() }

    pub fn to_hex(&self) -> String { hex::encode(self.to_bytes()) }

    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, PedersenError> {
        Ok(Self {
            point: CompressedRistretto::from_slice(bytes)
                .map_err(|_| PedersenError::InvalidPoint)?
                .decompress()
                .ok_or(PedersenError::InvalidPoint)?,
        })
    }

    pub fn from_hex(s: &str) -> Result<Self, PedersenError> {
        let bytes = hex::decode(s).map_err(|e| PedersenError::HexDecode(e.to_string()))?;
        Self::from_bytes(&bytes.try_into().map_err(|_| PedersenError::InvalidPoint)?)
    }

    pub fn add(&self, other: &Self) -> Self {
        Self { point: self.point + other.point }
    }
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct BlindingFactor(pub(crate) Scalar);

impl BlindingFactor {
    pub fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self { Self(Scalar::random(rng)) }

    pub fn from_bytes(bytes: &[u8; 32]) -> Option<Self> {
        Scalar::from_canonical_bytes(*bytes).into_option().map(Self)
    }

    pub fn to_bytes(&self) -> [u8; 32] { self.0.to_bytes() }
    pub fn to_hex(&self) -> String { hex::encode(self.to_bytes()) }

    pub fn from_hex(s: &str) -> Option<Self> {
        Self::from_bytes(&hex::decode(s).ok()?.try_into().ok()?)
    }
}

impl std::fmt::Debug for BlindingFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BlindingFactor([REDACTED])")
    }
}

pub mod compressed_point_serde {
    use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(p: &RistrettoPoint, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(p.compress().to_bytes()))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<RistrettoPoint, D::Error> {
        let bytes = hex::decode(String::deserialize(d)?).map_err(serde::de::Error::custom)?;
        CompressedRistretto::from_slice(
            &<[u8; 32]>::try_from(bytes)
                .map_err(|_| serde::de::Error::custom("expected 32 bytes"))?,
        )
        .map_err(serde::de::Error::custom)?
        .decompress()
        .ok_or_else(|| serde::de::Error::custom("invalid Ristretto255 point"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    #[test] fn generators_distinct() {
        let g = PedersenGenerators::standard();
        assert_ne!(g.g.compress(), g.h.compress());
    }
    #[test] fn generators_deterministic() {
        assert_eq!(PedersenGenerators::standard().h.compress(), PedersenGenerators::standard().h.compress());
    }
    #[test] fn generators_serde() {
        let g = PedersenGenerators::standard();
        assert_eq!(g, serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap());
    }
    #[test] fn commit_verify_correct() {
        let g = PedersenGenerators::standard();
        let r = BlindingFactor::random(&mut OsRng);
        let c = PedersenCommitment::commit(42_000, &r, &g);
        assert!(c.verify(42_000, &r, &g));
    }
    #[test] fn commit_verify_wrong_value() {
        let g = PedersenGenerators::standard();
        let r = BlindingFactor::random(&mut OsRng);
        assert!(!PedersenCommitment::commit(1_000, &r, &g).verify(1_001, &r, &g));
    }
    #[test] fn commit_verify_wrong_blinding() {
        let g = PedersenGenerators::standard();
        let (r1, r2) = (BlindingFactor::random(&mut OsRng), BlindingFactor::random(&mut OsRng));
        assert!(!PedersenCommitment::commit(100, &r1, &g).verify(100, &r2, &g));
    }
    #[test] fn hiding() {
        let g = PedersenGenerators::standard();
        let (r1, r2) = (BlindingFactor::random(&mut OsRng), BlindingFactor::random(&mut OsRng));
        assert_ne!(PedersenCommitment::commit(500, &r1, &g), PedersenCommitment::commit(500, &r2, &g));
    }
    #[test] fn bytes_roundtrip() {
        let g = PedersenGenerators::standard();
        let r = BlindingFactor::random(&mut OsRng);
        let c = PedersenCommitment::commit(999, &r, &g);
        assert_eq!(c, PedersenCommitment::from_bytes(&c.to_bytes()).unwrap());
    }
    #[test] fn hex_roundtrip() {
        let g = PedersenGenerators::standard();
        let r = BlindingFactor::random(&mut OsRng);
        let c = PedersenCommitment::commit(1234, &r, &g);
        assert_eq!(64, c.to_hex().len());
        assert_eq!(c, PedersenCommitment::from_hex(&c.to_hex()).unwrap());
    }
    #[test] fn json_roundtrip() {
        let g = PedersenGenerators::standard();
        let r = BlindingFactor::random(&mut OsRng);
        let c = PedersenCommitment::commit(77, &r, &g);
        assert_eq!(c, serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap());
    }
    #[test] fn invalid_point_bytes_err() {
        assert!(PedersenCommitment::from_bytes(&[0u8; 32]).is_err());
    }
    #[test] fn homomorphic_add() {
        let g = PedersenGenerators::standard();
        let (r1, r2) = (BlindingFactor::random(&mut OsRng), BlindingFactor::random(&mut OsRng));
        let r_sum = BlindingFactor(r1.0 + r2.0);
        let sum = PedersenCommitment::commit(30, &r1, &g).add(&PedersenCommitment::commit(70, &r2, &g));
        assert!(sum.verify(100, &r_sum, &g));
    }
    #[test] fn blinding_hex_roundtrip() {
        let r = BlindingFactor::random(&mut OsRng);
        assert_eq!(r.to_bytes(), BlindingFactor::from_hex(&r.to_hex()).unwrap().to_bytes());
    }
    #[test] fn debug_redacted() {
        assert!(format!("{:?}", BlindingFactor::random(&mut OsRng)).contains("[REDACTED]"));
    }
}