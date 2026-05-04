use uuid::Uuid;
use auction_crypto::{
    pedersen::PedersenGenerators,
    signature::ServerVerifier,
};
use auction_verifier::{
    bulletin_board::{verify_chain_integrity, verify_chain_with_signatures},
    transcript::{verify_auction_transcript, AuctionTranscript, LoserData, WinnerData, VerificationResult},
};
use crate::{client::AuctionClient, errors::ClientError};

pub struct ClientVerifier {
    server_verifier: ServerVerifier,
    pedersen_generators: PedersenGenerators,
}

impl ClientVerifier {
    pub fn new(server_verifier: ServerVerifier, pedersen_generators: PedersenGenerators) -> Self {
        Self { server_verifier, pedersen_generators }
    }

    pub async fn from_server(client: &AuctionClient) -> Result<Self, ClientError> {
        let info = client.get_server_public_key().await?;
        let pk_bytes = hex::decode(&info.public_key_hex)
            .map_err(|e| ClientError::Hex(e.to_string()))?;
        let arr: [u8; 32] = pk_bytes.try_into()
            .map_err(|_| ClientError::Internal("invalid pubkey len".into()))?;
        let verifier = ServerVerifier::from_bytes(&arr)
            .map_err(|e| ClientError::Crypto(e.to_string()))?;
        Ok(Self { server_verifier: verifier, pedersen_generators: PedersenGenerators::standard() })
    }

    pub async fn verify_auction(
        &self,
        client: &AuctionClient,
        auction_id: Uuid,
    ) -> Result<VerificationResult, ClientError> {
        let bb = client.get_bulletin_board(auction_id).await?;
        let winner_detail = client.get_winner_reveal(auction_id).await.ok();
        let losers_resp = client.list_loser_proofs(auction_id).await?;
        let bids_resp = client.list_bids(auction_id).await?;

        let winner = winner_detail.map(|w| {
            let bid = bids_resp.bids.iter().find(|b| b.bid_id == w.bid_id);
            WinnerData {
                bidder_id: w.winner_id,
                bid_id: w.bid_id,
                commitment_hex: bid.map(|b| b.commitment_hex.clone()).unwrap_or_default(),
                revealed_value: w.revealed_value as u64,
                proof_json: w.proof_json,
            }
        });

        let losers = losers_resp.proofs.iter().map(|p| {
            let bid = bids_resp.bids.iter().find(|b| b.bid_id == p.bid_id);
            LoserData {
                bidder_id: p.bidder_id,
                bid_id: p.bid_id,
                commitment_hex: bid.map(|b| b.commitment_hex.clone()).unwrap_or_default(),
                revealed_value: p.revealed_value as u64,
                proof_json: p.proof_json.clone(),
            }
        }).collect();

        let auction = client.get_auction(auction_id).await?; 

        let bb = client.get_bulletin_board(auction_id).await?;
        let winner_detail = client.get_winner_reveal(auction_id).await.ok();

        let transcript = AuctionTranscript {
            auction_id,
            min_bid: auction.min_bid as u64,                    // <-- AGGIUNTO
            max_bid: auction.max_bid.map(|m| m as u64),         // <-- AGGIUNTO
            bid_step: auction.bid_step as u64,
            bulletin_board: bb.entries,
            winner,
            losers,
            server_verifier: self.server_verifier.clone(),
            pedersen_generators: self.pedersen_generators.clone(),
        };

        Ok(verify_auction_transcript(&transcript))
    }

    pub fn verify_chain_only(
        &self,
        entries: &[auction_core::bulletin_board::BulletinBoardEntry],
    ) -> Result<(), auction_verifier::bulletin_board::ChainVerifyError> {
        verify_chain_with_signatures(entries, &self.server_verifier)
    }

    pub fn verify_my_bid(
        &self,
        secret: &crate::bid::BidSecret,
    ) -> bool {
        crate::bid::verify_my_commitment(secret, &self.pedersen_generators)
    }
}