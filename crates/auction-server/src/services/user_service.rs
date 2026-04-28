use argon2::{password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use sqlx::SqlitePool;
use uuid::Uuid;
use auction_core::{errors::AuctionError, responses::LoginResponse, user::User};
use crate::{auth::jwt::create_token, storage::user_repo::UserRepo};

pub struct UserService {
    repo: UserRepo,
    jwt_secret: String,
}

impl UserService {
    pub fn new(pool: SqlitePool, jwt_secret: String) -> Self {
        Self { repo: UserRepo(pool), jwt_secret }
    }

    pub async fn register(
        &self, username: String, password: String, public_key_hex: String,
    ) -> Result<User, AuctionError> {
        if self.repo.username_exists(&username).await? {
            return Err(AuctionError::UsernameTaken);
        }
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuctionError::Internal(e.to_string()))?
            .to_string();
        let user = User::new(username, hash, public_key_hex);
        self.repo.insert(&user).await?;
        Ok(user)
    }

    pub async fn login(&self, username: String, password: String) -> Result<LoginResponse, AuctionError> {
        let user = self.repo.find_by_username(&username).await?
            .ok_or(AuctionError::InvalidCredentials)?;
        let parsed = PasswordHash::new(&user.password_hash)
            .map_err(|e| AuctionError::Internal(e.to_string()))?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| AuctionError::InvalidCredentials)?;
        let token = create_token(user.id, &self.jwt_secret)
            .map_err(|e| AuctionError::Internal(e.to_string()))?;
        Ok(LoginResponse {
            jwt_token: token,
            user_id: user.id,
            username: user.username.clone(),
            public_key_hex: user.public_key_hex.clone(),
        })
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<User, AuctionError> {
        self.repo.find_by_id(id).await
    }
}