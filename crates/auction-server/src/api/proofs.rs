use std::sync::Arc;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;
use auction_core::{
    enums::AuctionStatus,
    errors::AuctionError,
    requests::{RevealWinnerRequest, SubmitLoserProofRequest},
    responses::{LoserProofListResponse, LoserProofResponse, RevealWinnerResponse, WinnerRevealDetailResponse},
};
use crate::{
    auth::middleware::AuthUser,
    errors::ApiResult,
    models::bb_payloads::{LoserProofPayload, WinnerRevealPayload},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auctions/:id/reveal",       post(reveal_winner).get(get_winner_reveal))
        .route("/auctions/:id/loser-proofs", post(submit_loser_proof).get(list_loser_proofs))
}

async fn reveal_winner(
    State(state): State<Arc<AppState>>,
    AuthUser(user_id): AuthUser,
    Path(auction_id): Path<Uuid>,
    Json(req): Json<RevealWinnerRequest>,
) -> ApiResult<Json<RevealWinnerResponse>> {
    state.auction_service.require_status(auction_id, &AuctionStatus::ClaimPhase).await?;

    let bid = state.bid_service.get_by_id(req.bid_id).await?;
    if bid.auction_id != auction_id {
        return Err(AuctionError::BidderNotInAuction(user_id, auction_id).into());
    }
    if bid.bidder_id != user_id {
        return Err(AuctionError::NotCreator.into());
    }

    let record = state.proof_service.submit_winner_reveal(
        auction_id,
        user_id,
        bid.id,
        req.revealed_value,
        req.proof_json.clone(),
        &bid.commitment_hex,
        &state.pedersen_generators,
    ).await?;

    let payload = serde_json::to_value(WinnerRevealPayload {
        reveal_id: record.id,
        auction_id,
        winner_id: user_id,
        bid_id: bid.id,
        revealed_value: record.revealed_value,
        proof_json: req.proof_json,
    })?;
    let entry = state.bulletin_board_service
        .append(auction_id, auction_core::bulletin_board::EntryKind::WinnerReveal, payload, &state.server_signer)
        .await?;
    state.proof_service.update_winner_bb_sequence(record.id, entry.sequence).await?;
    state.auction_service.transition(auction_id, user_id, AuctionStatus::ProofPhase).await.ok();

    Ok(Json(RevealWinnerResponse {
        reveal_id: record.id,
        winner_id: user_id,
        revealed_value: record.revealed_value,
        bb_entry_hash_hex: entry.entry_hash_hex,
        bb_sequence: Some(entry.sequence),
    }))
}

async fn get_winner_reveal(
    State(state): State<Arc<AppState>>,
    Path(auction_id): Path<Uuid>,
) -> ApiResult<Json<WinnerRevealDetailResponse>> {
    let record = state.proof_service.get_winner_reveal(auction_id).await?
        .ok_or_else(|| AuctionError::Internal("no winner reveal yet".into()))?;
    Ok(Json(WinnerRevealDetailResponse::from(record)))
}

async fn submit_loser_proof(
    State(state): State<Arc<AppState>>,
    AuthUser(user_id): AuthUser,
    Path(auction_id): Path<Uuid>,
    Json(req): Json<SubmitLoserProofRequest>,
) -> ApiResult<Json<LoserProofResponse>> {
    state.auction_service.require_status(auction_id, &AuctionStatus::ProofPhase).await?;

    let bid = state.bid_service.get_by_id(req.bid_id).await?;
    if bid.bidder_id != user_id || bid.auction_id != auction_id {
        return Err(AuctionError::BidderNotInAuction(user_id, auction_id).into());
    }

    let winner = state.proof_service.get_winner_reveal(auction_id).await?
        .ok_or_else(|| AuctionError::Internal("no winner reveal".into()))?;

    let record = state.proof_service.submit_loser_proof(
        auction_id,
        user_id,
        bid.id,
        req.revealed_value,
        req.proof_json.clone(),
        &bid.commitment_hex,
        winner.revealed_value,
        &state.pedersen_generators,
    ).await?;

    let payload = serde_json::to_value(LoserProofPayload {
        proof_id: record.id,
        auction_id,
        bidder_id: user_id,
        bid_id: bid.id,
        revealed_value: record.revealed_value,
    })?;
    let entry = state.bulletin_board_service
        .append(auction_id, auction_core::bulletin_board::EntryKind::LoserProof, payload, &state.server_signer)
        .await?;
    state.proof_service.update_loser_bb_sequence(record.id, entry.sequence).await?;

    Ok(Json(LoserProofResponse::from(record)))
}

async fn list_loser_proofs(
    State(state): State<Arc<AppState>>,
    Path(auction_id): Path<Uuid>,
) -> ApiResult<Json<LoserProofListResponse>> {
    let proofs = state.proof_service.get_loser_proofs(auction_id).await?;
    let total = proofs.len();
    Ok(Json(LoserProofListResponse {
        proofs: proofs.into_iter().map(LoserProofResponse::from).collect(),
        total,
    }))
}