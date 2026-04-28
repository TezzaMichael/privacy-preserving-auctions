use ed25519_dalek::{
    ed25519::signature::Signer as DalekSigner, Signature, SigningKey,
    Verifier as DalekVerifier, VerifyingKey,
};
use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::ZeroizeOnDrop;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SignatureError {
    #[error("Ed25519 signature verification failed")]
    VerificationFailed,
    #[error("invalid public key bytes")]
    InvalidPublicKey,
    #[error("invalid signature bytes (expected 64)")]
    InvalidSignature,
}

#[derive(ZeroizeOnDrop)]
pub struct ServerSigner {
    signing_key: SigningKey,
}

impl ServerSigner {
    pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        Self { signing_key: SigningKey::generate(rng) }
    }
    pub fn from_bytes(b: &[u8; 32]) -> Self { Self { signing_key: SigningKey::from_bytes(b) } }
    pub fn sign(&self, msg: &[u8]) -> Vec<u8> { self.signing_key.sign(msg).to_bytes().to_vec() }
    pub fn verifier(&self) -> ServerVerifier { ServerVerifier { verifying_key: self.signing_key.verifying_key() } }
    pub fn to_secret_bytes(&self) -> [u8; 32] { self.signing_key.to_bytes() }
}

impl std::fmt::Debug for ServerSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("ServerSigner([SECRET])") }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerVerifier {
    #[serde(with = "vk_serde")]
    pub verifying_key: VerifyingKey,
}

impl ServerVerifier {
    pub fn from_bytes(b: &[u8; 32]) -> Result<Self, SignatureError> {
        VerifyingKey::from_bytes(b).map(|vk| Self { verifying_key: vk }).map_err(|_| SignatureError::InvalidPublicKey)
    }
    pub fn verify(&self, msg: &[u8], sig_bytes: &[u8]) -> Result<(), SignatureError> {
        let arr: [u8; 64] = sig_bytes.try_into().map_err(|_| SignatureError::InvalidSignature)?;
        self.verifying_key.verify(msg, &Signature::from_bytes(&arr)).map_err(|_| SignatureError::VerificationFailed)
    }
    pub fn to_bytes(&self) -> [u8; 32] { self.verifying_key.to_bytes() }
    pub fn to_hex(&self) -> String { hex::encode(self.to_bytes()) }
}

mod vk_serde {
    use ed25519_dalek::VerifyingKey;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(vk: &VerifyingKey, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(vk.to_bytes()))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<VerifyingKey, D::Error> {
        VerifyingKey::from_bytes(
            &<[u8; 32]>::try_from(
                hex::decode(String::deserialize(d)?).map_err(serde::de::Error::custom)?,
            )
            .map_err(|_| serde::de::Error::custom("expected 32 bytes"))?,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    fn kp() -> ServerSigner { ServerSigner::generate(&mut OsRng) }

    #[test] fn sign_verify() { let s = kp(); assert!(s.verifier().verify(b"msg", &s.sign(b"msg")).is_ok()); }
    #[test] fn tampered_msg() { let s = kp(); assert_eq!(s.verifier().verify(b"x",&s.sign(b"y")), Err(SignatureError::VerificationFailed)); }
    #[test] fn tampered_sig() { let s = kp(); let mut sig = s.sign(b"m"); sig[0]^=0xFF; assert!(s.verifier().verify(b"m",&sig).is_err()); }
    #[test] fn wrong_key() { let s1=kp(); let s2=kp(); assert!(s2.verifier().verify(b"m",&s1.sign(b"m")).is_err()); }
    #[test] fn short_sig() { let s=kp(); assert_eq!(s.verifier().verify(b"m",&[0u8;32]), Err(SignatureError::InvalidSignature)); }
    #[test] fn secret_roundtrip() {
        let s1=kp(); let s2=ServerSigner::from_bytes(&s1.to_secret_bytes());
        assert_eq!(s1.verifier().to_bytes(), s2.verifier().to_bytes());
    }
    #[test] fn verifier_bytes_roundtrip() {
        let s=kp(); let v=ServerVerifier::from_bytes(&s.verifier().to_bytes()).unwrap();
        assert!(v.verify(b"test",&s.sign(b"test")).is_ok());
    }
    #[test] fn verifier_json_roundtrip() {
        let s=kp(); let v=s.verifier();
        let v2: ServerVerifier = serde_json::from_str(&serde_json::to_string(&v).unwrap()).unwrap();
        assert_eq!(v,v2); assert!(v2.verify(b"x",&s.sign(b"x")).is_ok());
    }
    #[test] fn debug_redacted() { assert!(format!("{:?}",kp()).contains("[SECRET]")); }
}