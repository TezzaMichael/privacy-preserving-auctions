use auction_crypto::{
    pedersen::{BlindingFactor, PedersenCommitment, PedersenGenerators},
    schnorr::ProofOfOpening,
};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::errors::ClientError;

#[derive(Debug, Serialize, Deserialize)]
pub struct BidSecret {
    pub auction_id: Uuid,
    pub value: u64,
    pub blinding_hex: String,
    pub commitment_hex: String,
}

#[derive(Debug)]
pub struct SealedBidData {
    pub commitment_hex: String,
    pub bidder_signature_hex: String,
    pub secret: BidSecret,
}

pub fn create_sealed_bid(
    auction_id: Uuid,
    value: u64,
    gens: &PedersenGenerators,
    identity: &crate::identity::Identity,
) -> SealedBidData {
    let blinding = BlindingFactor::random(&mut OsRng);
    let commitment = PedersenCommitment::commit(value, &blinding, gens);
    let commitment_hex = commitment.to_hex();
    let sig = identity.sign_commitment(auction_id.as_bytes(), &commitment_hex);
    let secret = BidSecret {
        auction_id,
        value,
        blinding_hex: blinding.to_hex(),
        commitment_hex: commitment_hex.clone(),
    };
    SealedBidData {
        commitment_hex,
        bidder_signature_hex: hex::encode(sig),
        secret,
    }
}

pub fn create_proof_of_opening(
    secret: &BidSecret,
    gens: &PedersenGenerators,
) -> Result<ProofOfOpening, ClientError> {
    let blinding = BlindingFactor::from_hex(&secret.blinding_hex)
        .ok_or_else(|| ClientError::Crypto("invalid blinding hex".into()))?;
    let commitment = PedersenCommitment::from_hex(&secret.commitment_hex)
        .map_err(|e| ClientError::Crypto(e.to_string()))?;
    Ok(ProofOfOpening::prove(secret.value, &blinding, &commitment, gens, &mut OsRng))
}

pub fn verify_my_commitment(secret: &BidSecret, gens: &PedersenGenerators) -> bool {
    let Ok(blinding) = BlindingFactor::from_hex(&secret.blinding_hex)
        .ok_or(()) else { return false; };
    let Ok(commitment) = PedersenCommitment::from_hex(&secret.commitment_hex) else { return false; };
    commitment.verify(secret.value, &blinding, gens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::Identity;

    #[test] fn sealed_bid_roundtrip() {
        let gens = PedersenGenerators::standard();
        let id = Identity::generate("alice".into());
        let auction_id = Uuid::new_v4();
        let data = create_sealed_bid(auction_id, 1000, &gens, &id);
        assert!(verify_my_commitment(&data.secret, &gens));
        let proof = create_proof_of_opening(&data.secret, &gens).unwrap();
        assert!(proof.verify(&gens).is_ok());
        assert_eq!(proof.revealed_value, 1000);
    }

    #[test] fn commitment_verify_wrong_value_fails() {
        let gens = PedersenGenerators::standard();
        let id = Identity::generate("bob".into());
        let mut data = create_sealed_bid(Uuid::new_v4(), 500, &gens, &id);
        data.secret.value = 501;
        assert!(!verify_my_commitment(&data.secret, &gens));
    }
}