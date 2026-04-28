use ed25519_dalek::{
    ed25519::signature::Signer as DalekSigner, Signature, SigningKey,
    Verifier as DalekVerifier, VerifyingKey,
};
use rand::{CryptoRng, RngCore};
use sha2::{Digest, Sha256};
use zeroize::ZeroizeOnDrop;

#[derive(ZeroizeOnDrop)]
pub struct BidderKeyPair {
    signing_key: SigningKey,
}

impl BidderKeyPair {
    pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        Self { signing_key: SigningKey::generate(rng) }
    }
    pub fn from_bytes(b: &[u8; 32]) -> Self { Self { signing_key: SigningKey::from_bytes(b) } }
    pub fn public_key_bytes(&self) -> [u8; 32] { self.signing_key.verifying_key().to_bytes() }
    pub fn public_key_hex(&self) -> String { hex::encode(self.public_key_bytes()) }
    pub fn to_secret_bytes(&self) -> [u8; 32] { self.signing_key.to_bytes() }

    pub fn sign_commitment(&self, auction_id: &[u8], commitment_hex: &str) -> Vec<u8> {
        self.signing_key.sign(&commitment_message(auction_id, commitment_hex)).to_bytes().to_vec()
    }
    pub fn sign_raw(&self, msg: &[u8]) -> Vec<u8> { self.signing_key.sign(msg).to_bytes().to_vec() }
}

impl std::fmt::Debug for BidderKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BidderKeyPair {{ public_key: {} }}", self.public_key_hex())
    }
}

pub fn verify_commitment_signature(
    pk_hex: &str,
    auction_id: &[u8],
    commitment_hex: &str,
    sig_bytes: &[u8],
) -> bool {
    let Some(vk) = decode_vk(pk_hex) else { return false; };
    let Ok(arr) = <[u8; 64]>::try_from(sig_bytes) else { return false; };
    vk.verify(&commitment_message(auction_id, commitment_hex), &Signature::from_bytes(&arr)).is_ok()
}

pub fn verify_raw_signature(pk_hex: &str, msg: &[u8], sig_bytes: &[u8]) -> bool {
    let Some(vk) = decode_vk(pk_hex) else { return false; };
    let Ok(arr) = <[u8; 64]>::try_from(sig_bytes) else { return false; };
    vk.verify(msg, &Signature::from_bytes(&arr)).is_ok()
}

fn decode_vk(pk_hex: &str) -> Option<VerifyingKey> {
    VerifyingKey::from_bytes(&hex::decode(pk_hex).ok()?.try_into().ok()?).ok()
}

fn commitment_message(auction_id: &[u8], commitment_hex: &str) -> Vec<u8> {
    let ch = commitment_hex.as_bytes();
    let mut h = Sha256::new();
    h.update(b"auction-bid-commitment-v1:");
    h.update((auction_id.len() as u64).to_le_bytes());
    h.update(auction_id);
    h.update((ch.len() as u64).to_le_bytes());
    h.update(ch);
    h.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;
    use uuid::Uuid;

    fn kp() -> BidderKeyPair { BidderKeyPair::generate(&mut OsRng) }
    fn aid() -> Vec<u8> { Uuid::new_v4().as_bytes().to_vec() }
    fn comm() -> String { "a3b2c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2".to_string() }

    #[test] fn sign_verify_commitment() {
        let k=kp(); let a=aid(); let c=comm();
        assert!(verify_commitment_signature(&k.public_key_hex(),&a,&c,&k.sign_commitment(&a,&c)));
    }
    #[test] fn sign_verify_raw() {
        let k=kp(); assert!(verify_raw_signature(&k.public_key_hex(),b"msg",&k.sign_raw(b"msg")));
    }
    #[test] fn wrong_auction_id_fails() {
        let k=kp(); let c=comm();
        let sig=k.sign_commitment(&aid(),&c);
        assert!(!verify_commitment_signature(&k.public_key_hex(),&aid(),&c,&sig));
    }
    #[test] fn wrong_commitment_fails() {
        let k=kp(); let a=aid();
        let sig=k.sign_commitment(&a,&comm());
        assert!(!verify_commitment_signature(&k.public_key_hex(),&a,"0000000000000000000000000000000000000000000000000000000000000001",&sig));
    }
    #[test] fn wrong_key_fails() {
        let (k1,k2)=(kp(),kp()); let a=aid(); let c=comm();
        assert!(!verify_commitment_signature(&k2.public_key_hex(),&a,&c,&k1.sign_commitment(&a,&c)));
    }
    #[test] fn tampered_sig_fails() {
        let k=kp(); let a=aid(); let c=comm();
        let mut sig=k.sign_commitment(&a,&c); sig[0]^=0xFF;
        assert!(!verify_commitment_signature(&k.public_key_hex(),&a,&c,&sig));
    }
    #[test] fn short_sig_fails() {
        let k=kp(); let a=aid(); let c=comm();
        assert!(!verify_commitment_signature(&k.public_key_hex(),&a,&c,&[0u8;32]));
    }
    #[test] fn invalid_pk_hex_fails() {
        let k=kp(); let a=aid(); let c=comm();
        assert!(!verify_commitment_signature("not-hex",&a,&c,&k.sign_commitment(&a,&c)));
    }
    #[test] fn secret_roundtrip() {
        let k1=kp(); let k2=BidderKeyPair::from_bytes(&k1.to_secret_bytes());
        assert_eq!(k1.public_key_bytes(),k2.public_key_bytes());
    }
    #[test] fn domain_separates_from_raw() {
        let k=kp(); let a=aid(); let c=comm();
        assert!(!verify_raw_signature(&k.public_key_hex(),c.as_bytes(),&k.sign_commitment(&a,&c)));
    }
    #[test] fn debug_shows_pubkey() {
        let k=kp(); assert!(format!("{k:?}").contains("public_key"));
    }
}