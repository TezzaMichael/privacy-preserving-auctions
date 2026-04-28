use sqlx::SqlitePool;
use uuid::Uuid;
use auction_core::{errors::AuctionError, proof::{LoserProofRecord, WinnerRevealRecord}};
use auction_crypto::{pedersen::PedersenGenerators, schnorr::ProofOfOpening};
use crate::storage::proof_repo::ProofRepo;

pub struct ProofService {
    repo: ProofRepo,
}

impl ProofService {
    pub fn new(pool: SqlitePool) -> Self { Self { repo: ProofRepo(pool) } }

    pub async fn submit_winner_reveal(
        &self,
        auction_id: Uuid,
        winner_id: Uuid,
        bid_id: Uuid,
        revealed_value: u64,
        proof_json: String,
        stored_commitment_hex: &str,
        gens: &PedersenGenerators,
    ) -> Result<WinnerRevealRecord, AuctionError> {
        if self.repo.find_winner_reveal(auction_id).await?.is_some() {
            return Err(AuctionError::RevealAlreadySubmitted);
        }
        let proof: ProofOfOpening = serde_json::from_str(&proof_json)?;
        if proof.revealed_value != revealed_value {
            return Err(AuctionError::InvalidProof);
        }
        let stored = auction_crypto::pedersen::PedersenCommitment::from_hex(stored_commitment_hex)
            .map_err(|_| AuctionError::InvalidCommitment)?;
        use subtle::ConstantTimeEq;
        if !bool::from(proof.commitment.compress().as_bytes().ct_eq(&stored.to_bytes())) {
            return Err(AuctionError::InvalidProof);
        }
        proof.verify(gens).map_err(|_| AuctionError::InvalidProof)?;
        let record = WinnerRevealRecord::new(
            auction_id, winner_id, bid_id, revealed_value as i64, proof_json,
        );
        self.repo.insert_winner_reveal(&record).await?;
        Ok(record)
    }

    pub async fn submit_loser_proof(
        &self,
        auction_id: Uuid,
        bidder_id: Uuid,
        bid_id: Uuid,
        revealed_value: u64,
        proof_json: String,
        stored_commitment_hex: &str,
        winner_value: i64,
        gens: &PedersenGenerators,
    ) -> Result<LoserProofRecord, AuctionError> {
        if self.repo.find_loser_proof_by_bidder(auction_id, bidder_id).await?.is_some() {
            return Err(AuctionError::DuplicateProof);
        }
        if revealed_value as i64 >= winner_value {
            return Err(AuctionError::NotALoser);
        }
        let proof: ProofOfOpening = serde_json::from_str(&proof_json)?;
        if proof.revealed_value != revealed_value {
            return Err(AuctionError::InvalidProof);
        }
        let stored = auction_crypto::pedersen::PedersenCommitment::from_hex(stored_commitment_hex)
            .map_err(|_| AuctionError::InvalidCommitment)?;
        use subtle::ConstantTimeEq;
        if !bool::from(proof.commitment.compress().as_bytes().ct_eq(&stored.to_bytes())) {
            return Err(AuctionError::InvalidProof);
        }
        proof.verify(gens).map_err(|_| AuctionError::InvalidProof)?;
        let mut record = LoserProofRecord::new(
            auction_id, bidder_id, bid_id, revealed_value as i64, proof_json,
        );
        record.verified = true;
        self.repo.insert_loser_proof(&record).await?;
        Ok(record)
    }

    pub async fn get_winner_reveal(&self, auction_id: Uuid) -> Result<Option<WinnerRevealRecord>, AuctionError> {
        self.repo.find_winner_reveal(auction_id).await
    }

    pub async fn get_loser_proofs(&self, auction_id: Uuid) -> Result<Vec<LoserProofRecord>, AuctionError> {
        self.repo.find_loser_proofs(auction_id).await
    }

    pub async fn update_winner_bb_sequence(&self, id: Uuid, seq: i64) -> Result<(), AuctionError> {
        self.repo.update_winner_bb_sequence(id, seq).await
    }

    pub async fn update_loser_bb_sequence(&self, id: Uuid, seq: i64) -> Result<(), AuctionError> {
        self.repo.update_loser_bb_sequence(id, seq).await
    }
}