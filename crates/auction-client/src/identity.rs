use auction_crypto::keys::BidderKeyPair;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct IdentityFile {
    pub username: String,
    pub secret_key_hex: String,
}

pub struct Identity {
    pub username: String,
    pub keypair: BidderKeyPair,
}

impl Identity {
    pub fn generate(username: String) -> Self {
        Self { username, keypair: BidderKeyPair::generate(&mut OsRng) }
    }

    pub fn from_file(f: IdentityFile) -> Result<Self, crate::errors::ClientError> {
        let bytes = hex::decode(&f.secret_key_hex)
            .map_err(|e| crate::errors::ClientError::Hex(e.to_string()))?;
        let arr: [u8; 32] = bytes.try_into()
            .map_err(|_| crate::errors::ClientError::Internal("invalid key length".into()))?;
        Ok(Self { username: f.username, keypair: BidderKeyPair::from_bytes(&arr) })
    }

    pub fn to_file(&self) -> IdentityFile {
        IdentityFile {
            username: self.username.clone(),
            secret_key_hex: hex::encode(self.keypair.to_secret_bytes()),
        }
    }

    pub fn public_key_hex(&self) -> String {
        self.keypair.public_key_hex()
    }

    pub fn sign_commitment(&self, auction_id_bytes: &[u8], commitment_hex: &str) -> Vec<u8> {
        self.keypair.sign_commitment(auction_id_bytes, commitment_hex)
    }
}