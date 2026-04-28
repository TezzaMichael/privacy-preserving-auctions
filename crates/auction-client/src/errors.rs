use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("api error {code}: {message}")]
    Api { code: u16, message: String },
    #[error("crypto error: {0}")]
    Crypto(String),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("verification failed: {0}")]
    Verification(String),
    #[error("no stored bid secret for auction {0}")]
    NoSecret(uuid::Uuid),
    #[error("invalid hex: {0}")]
    Hex(String),
    #[error("internal: {0}")]
    Internal(String),
}