use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use uuid::Uuid;

use crate::models::{
    auction::Auction,
    bid::Bid,
    bulletin_board::BulletinEntry,
    proof::ProofCertificate,
    reveal::WinnerReveal,
    user::User,
};

#[derive(Debug, Clone)]
pub struct AppState {
    pub users: Arc<RwLock<HashMap<Uuid, User>>>,
    pub by_name: Arc<RwLock<HashMap<String, Uuid>>>,
    pub auctions: Arc<RwLock<HashMap<Uuid, Auction>>>,
    pub bids: Arc<RwLock<HashMap<(Uuid, Uuid), Bid>>>,
    pub reveals: Arc<RwLock<HashMap<Uuid, WinnerReveal>>>,
    pub proofs: Arc<RwLock<HashMap<(Uuid, Uuid), ProofCertificate>>>,
    pub bb: Arc<RwLock<HashMap<Uuid, Vec<BulletinEntry>>>>,
    pub jwt_secret: String,
    pub server_private_key: String,
    pub server_public_key: String,
    pub pedersen_b: String,
    pub pedersen_b_blind: String,
}