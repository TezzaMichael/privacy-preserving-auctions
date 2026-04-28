use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::enums::UserRole;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub public_key: String,
    pub role: UserRole,
    pub registered_at: DateTime<Utc>,
}