use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use uuid::Uuid;
use auction_core::{
    requests::{CreateAuctionRequest, LoginRequest, RegisterRequest, RevealWinnerRequest, SubmitBidRequest, SubmitLoserProofRequest},
    responses::{
        AuctionListResponse, AuctionResponse, BidListResponse, BulletinBoardResponse,
        LoginResponse, MeResponse, RegisterResponse, RevealWinnerResponse, ServerPublicKeyResponse,
        SubmitBidResponse, WinnerRevealDetailResponse, LoserProofListResponse, LoserProofResponse,
    },
};
use crate::errors::ClientError;

pub struct AuctionClient {
    http: Client,
    base_url: String,
    token: Option<String>,
}

impl AuctionClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self { http: Client::new(), base_url: base_url.into(), token: None }
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token); self
    }

    pub fn set_token(&mut self, token: String) { self.token = Some(token); }

    fn url(&self, path: &str) -> String { format!("{}{}", self.base_url, path) }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ClientError> {
        let mut req = self.http.get(self.url(path));
        if let Some(t) = &self.token { req = req.bearer_auth(t); }
        let resp = req.send().await?;
        self.parse(resp).await
    }

    async fn post<B: serde::Serialize, T: DeserializeOwned>(
        &self, path: &str, body: &B,
    ) -> Result<T, ClientError> {
        let mut req = self.http.post(self.url(path)).json(body);
        if let Some(t) = &self.token { req = req.bearer_auth(t); }
        let resp = req.send().await?;
        self.parse(resp).await
    }

    async fn parse<T: DeserializeOwned>(&self, resp: reqwest::Response) -> Result<T, ClientError> {
        let status = resp.status();
        if status.is_success() {
            Ok(resp.json::<T>().await?)
        } else {
            let code = status.as_u16();
            let message = resp.text().await.unwrap_or_else(|_| "unknown error".into());
            Err(ClientError::Api { code, message })
        }
    }

    pub async fn register(&self, username: &str, password: &str, public_key_hex: &str) -> Result<RegisterResponse, ClientError> {
        self.post("/auth/register", &RegisterRequest {
            username: username.into(),
            password: password.into(),
            public_key_hex: public_key_hex.into(),
        }).await
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<LoginResponse, ClientError> {
        self.post("/auth/login", &LoginRequest {
            username: username.into(),
            password: password.into(),
        }).await
    }

    pub async fn me(&self) -> Result<MeResponse, ClientError> {
        self.get("/auth/me").await
    }

    pub async fn create_auction(
        &self, title: &str, description: &str, min_bid: u64, max_bid: Option<u64>, step: u64, duration_seconds: i64,
    ) -> Result<AuctionResponse, ClientError> {
        self.post("/auctions", &CreateAuctionRequest {
            title: title.into(),
            description: description.into(),
            min_bid,
            max_bid,
            step,
            duration_seconds,
        }).await
    }

    pub async fn list_auctions(&self) -> Result<AuctionListResponse, ClientError> {
        self.get("/auctions").await
    }

    pub async fn get_auction(&self, id: Uuid) -> Result<AuctionResponse, ClientError> {
        self.get(&format!("/auctions/{id}")).await
    }

    pub async fn open_auction(&self, id: Uuid) -> Result<AuctionResponse, ClientError> {
        self.post(&format!("/auctions/{id}/open"), &serde_json::Value::Null).await
    }

    pub async fn close_auction(&self, id: Uuid) -> Result<AuctionResponse, ClientError> {
        self.post(&format!("/auctions/{id}/close"), &serde_json::Value::Null).await
    }

    pub async fn finalize_auction(&self, id: Uuid) -> Result<AuctionResponse, ClientError> {
        self.post(&format!("/auctions/{id}/finalize"), &serde_json::Value::Null).await
    }

    pub async fn submit_bid(
        &self, auction_id: Uuid, commitment_hex: &str, bidder_signature_hex: &str,
    ) -> Result<SubmitBidResponse, ClientError> {
        self.post(&format!("/auctions/{auction_id}/bids"), &SubmitBidRequest {
            commitment_hex: commitment_hex.into(),
            bidder_signature_hex: bidder_signature_hex.into(),
        }).await
    }

    pub async fn list_bids(&self, auction_id: Uuid) -> Result<BidListResponse, ClientError> {
        self.get(&format!("/auctions/{auction_id}/bids")).await
    }

    pub async fn reveal_winner(
        &self, auction_id: Uuid, bid_id: Uuid, revealed_value: u64, proof_json: &str,
    ) -> Result<RevealWinnerResponse, ClientError> {
        self.post(&format!("/auctions/{auction_id}/reveal"), &RevealWinnerRequest {
            bid_id,
            revealed_value,
            proof_json: proof_json.into(),
        }).await
    }

    pub async fn get_winner_reveal(&self, auction_id: Uuid) -> Result<WinnerRevealDetailResponse, ClientError> {
        self.get(&format!("/auctions/{auction_id}/reveal")).await
    }

    pub async fn submit_loser_proof(
        &self, auction_id: Uuid, bid_id: Uuid, revealed_value: u64, proof_json: &str,
    ) -> Result<LoserProofResponse, ClientError> {
        self.post(&format!("/auctions/{auction_id}/loser-proofs"), &SubmitLoserProofRequest {
            bid_id,
            revealed_value,
            proof_json: proof_json.into(),
        }).await
    }

    pub async fn list_loser_proofs(&self, auction_id: Uuid) -> Result<LoserProofListResponse, ClientError> {
        self.get(&format!("/auctions/{auction_id}/loser-proofs")).await
    }

    pub async fn get_bulletin_board(&self, auction_id: Uuid) -> Result<BulletinBoardResponse, ClientError> {
        self.get(&format!("/bulletin-board/{auction_id}")).await
    }

    pub async fn get_server_public_key(&self) -> Result<ServerPublicKeyResponse, ClientError> {
        self.get("/server/public-key").await
    }
}