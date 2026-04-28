use std::sync::Arc;
use sqlx::SqlitePool;
use auction_crypto::{
    pedersen::PedersenGenerators,
    signature::{ServerSigner, ServerVerifier},
};
use crate::{
    config::Config,
    services::{
        auction_service::AuctionService,
        bid_service::BidService,
        bulletin_board_service::BulletinBoardService,
        proof_service::ProofService,
        user_service::UserService,
    },
};

pub struct AppState {
    pub pool: SqlitePool,
    pub jwt_secret: String,
    pub server_signer: ServerSigner,
    pub server_verifier: ServerVerifier,
    pub pedersen_generators: PedersenGenerators,
    pub user_service: UserService,
    pub auction_service: AuctionService,
    pub bid_service: BidService,
    pub bulletin_board_service: BulletinBoardService,
    pub proof_service: ProofService,
}

impl AppState {
    pub async fn new(pool: SqlitePool, cfg: &Config) -> anyhow::Result<Self> {
        let key_bytes = hex::decode(&cfg.server_signing_key_hex)?;
        let arr: [u8; 32] = key_bytes.try_into().map_err(|_| anyhow::anyhow!("invalid key len"))?;
        let server_signer = ServerSigner::from_bytes(&arr);
        let server_verifier = server_signer.verifier();
        let pedersen_generators = PedersenGenerators::standard();

        Ok(Self {
            user_service: UserService::new(pool.clone(), cfg.jwt_secret.clone()),
            auction_service: AuctionService::new(pool.clone()),
            bid_service: BidService::new(pool.clone()),
            bulletin_board_service: BulletinBoardService::new(pool.clone()),
            proof_service: ProofService::new(pool.clone()),
            pool,
            jwt_secret: cfg.jwt_secret.clone(),
            server_signer,
            server_verifier,
            pedersen_generators,
        })
    }
}