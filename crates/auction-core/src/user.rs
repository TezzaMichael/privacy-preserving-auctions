use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub public_key_hex: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(username: String, password_hash: String, public_key_hex: String) -> Self {
        Self { id: Uuid::new_v4(), username, password_hash, public_key_hex, created_at: Utc::now() }
    }
}