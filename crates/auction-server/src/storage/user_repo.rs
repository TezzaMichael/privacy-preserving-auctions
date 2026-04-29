use sqlx::SqlitePool;
use uuid::Uuid;
use auction_core::{errors::AuctionError, user::User};

pub struct UserRepo(pub SqlitePool);

impl UserRepo {
    pub async fn insert(&self, user: &User) -> Result<(), AuctionError> {
        sqlx::query!(
            "INSERT INTO users (id, username, password_hash, public_key_hex, created_at)
             VALUES (?, ?, ?, ?, ?)",
            user.id, user.username, user.password_hash, user.public_key_hex, user.created_at
        )
        .execute(&self.0)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<User, AuctionError> {
        sqlx::query_as!(
            User,
            "SELECT id as \"id!: Uuid\", username, password_hash, public_key_hex,
                    created_at as \"created_at: _\"
             FROM users WHERE id = ?",
            id
        )
        .fetch_one(&self.0)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AuctionError::UserNotFound(id),
            e => AuctionError::Storage(e),
        })
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, AuctionError> {
        sqlx::query_as!(
            User,
            "SELECT id as \"id!: Uuid\", username, password_hash, public_key_hex,
                    created_at as \"created_at: _\"
             FROM users WHERE username = ?",
            username
        )
        .fetch_optional(&self.0)
        .await
        .map_err(AuctionError::Storage)
    }

    pub async fn username_exists(&self, username: &str) -> Result<bool, AuctionError> {
        let row = sqlx::query!("SELECT COUNT(*) as cnt FROM users WHERE username = ?", username)
            .fetch_one(&self.0)
            .await?;
        Ok(row.cnt > 0)
    }
}