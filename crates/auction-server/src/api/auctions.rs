use std::sync::Arc;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;
use auction_core::{
    enums::AuctionStatus,
    requests::CreateAuctionRequest,
    responses::{AuctionListResponse, AuctionResponse},
};
use crate::{
    auth::middleware::AuthUser,
    errors::ApiResult,
    models::bb_payloads::{AuctionCreatePayload, AuctionStatusPayload},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auctions",              post(create_auction).get(list_auctions))
        .route("/auctions/:id",          get(get_auction))
        .route("/auctions/:id/open",     post(open_auction))
        .route("/auctions/:id/close",    post(close_auction))
        .route("/auctions/:id/finalize", post(finalize_auction))
}

async fn create_auction(
    State(state): State<Arc<AppState>>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateAuctionRequest>,
) -> ApiResult<Json<AuctionResponse>> {
    let auction = state.auction_service
        .create(
            user_id, 
            req.title.clone(), 
            req.description, 
            req.min_bid as i64, 
            req.max_bid.map(|m| m as i64), 
            req.step as i64, 
            req.duration_seconds
        )
        .await?;

    let payload = serde_json::to_value(AuctionCreatePayload {
        auction_id: auction.id,
        creator_id: auction.creator_id,
        title: auction.title.clone(),
        min_bid: req.min_bid,
        max_bid: req.max_bid,
        step: req.step,
        end_time: auction.end_time,
    })?;
    let entry = state.bulletin_board_service
        .append(auction.id, auction_core::bulletin_board::EntryKind::AuctionCreate, payload, &state.server_signer)
        .await?;
    state.auction_service.set_bb_sequence(auction.id, entry.sequence).await?;

    let sig = hex::encode(state.server_signer.sign(auction.id.as_bytes()));
    state.auction_service.set_server_signature(auction.id, &sig).await?;

    let updated = state.auction_service.get(auction.id).await?;
    Ok(Json(AuctionResponse::from(updated)))
}

async fn list_auctions(State(state): State<Arc<AppState>>) -> ApiResult<Json<AuctionListResponse>> {
    let auctions = state.auction_service.list().await?;
    let total = auctions.len();
    Ok(Json(AuctionListResponse {
        auctions: auctions.into_iter().map(AuctionResponse::from).collect(),
        total,
    }))
}

async fn get_auction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<AuctionResponse>> {
    let auction = state.auction_service.get(id).await?;
    Ok(Json(AuctionResponse::from(auction)))
}

async fn open_auction(
    State(state): State<Arc<AppState>>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<AuctionResponse>> {
    let auction = state.auction_service
        .transition(id, user_id, AuctionStatus::BiddingOpen)
        .await?;
    let payload = serde_json::to_value(AuctionStatusPayload {
        auction_id: id,
        new_status: "BiddingOpen".into(),
    })?;
    state.bulletin_board_service
        .append(id, auction_core::bulletin_board::EntryKind::AuctionOpen, payload, &state.server_signer)
        .await?;
    Ok(Json(AuctionResponse::from(auction)))
}

async fn close_auction(
    State(state): State<Arc<AppState>>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<AuctionResponse>> {
    let auction = state.auction_service
        .transition(id, user_id, AuctionStatus::ClaimPhase)
        .await?;
    let payload = serde_json::to_value(AuctionStatusPayload {
        auction_id: id,
        new_status: "ClaimPhase".into(),
    })?;
    state.bulletin_board_service
        .append(id, auction_core::bulletin_board::EntryKind::AuctionClose, payload, &state.server_signer)
        .await?;
    Ok(Json(AuctionResponse::from(auction)))
}

async fn finalize_auction(
    State(state): State<Arc<AppState>>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<AuctionResponse>> {
    let winner = state.proof_service.get_winner_reveal(id).await?
        .ok_or_else(|| auction_core::errors::AuctionError::Internal("no winner reveal".into()))?;
    let total = state.bid_service.count_by_auction(id).await?;
    let auction = state.auction_service
        .transition(id, user_id, AuctionStatus::Closed)
        .await?;
    let payload = serde_json::to_value(crate::models::bb_payloads::AuctionFinalizePayload {
        auction_id: id,
        winner_id: winner.winner_id,
        winner_value: winner.revealed_value,
        total_bids: total,
    })?;
    state.bulletin_board_service
        .append(id, auction_core::bulletin_board::EntryKind::AuctionFinalize, payload, &state.server_signer)
        .await?;
    Ok(Json(AuctionResponse::from(auction)))
}