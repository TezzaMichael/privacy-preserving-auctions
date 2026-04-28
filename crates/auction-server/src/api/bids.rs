use std::sync::Arc;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;
use auction_core::{
    enums::AuctionStatus,
    requests::SubmitBidRequest,
    responses::{BidListResponse, SealedBidResponse, SubmitBidResponse},
};
use crate::{
    auth::middleware::AuthUser,
    errors::ApiResult,
    models::bb_payloads::SealedBidPayload,
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auctions/:id/bids", post(submit_bid).get(list_bids))
}

async fn submit_bid(
    State(state): State<Arc<AppState>>,
    AuthUser(user_id): AuthUser,
    Path(auction_id): Path<Uuid>,
    Json(req): Json<SubmitBidRequest>,
) -> ApiResult<Json<SubmitBidResponse>> {
    state.auction_service.require_status(auction_id, &AuctionStatus::BiddingOpen).await?;
    let user = state.user_service.get_by_id(user_id).await?;

    let bid = state.bid_service.submit(
        auction_id,
        user_id,
        &user.public_key_hex,
        req.commitment_hex.clone(),
        req.bidder_signature_hex,
    ).await?;

    let payload = serde_json::to_value(SealedBidPayload {
        bid_id: bid.id,
        auction_id,
        bidder_id: user_id,
        commitment_hex: req.commitment_hex,
        bidder_signature_hex: bid.bidder_signature_hex.clone(),
    })?;
    let entry = state.bulletin_board_service
        .append(auction_id, auction_core::bulletin_board::EntryKind::SealedBid, payload, &state.server_signer)
        .await?;
    state.bid_service.update_bb_sequence(bid.id, entry.sequence).await?;

    Ok(Json(SubmitBidResponse {
        bid_id: bid.id,
        bb_entry_hash_hex: entry.entry_hash_hex,
        bb_sequence: entry.sequence,
    }))
}

async fn list_bids(
    State(state): State<Arc<AppState>>,
    Path(auction_id): Path<Uuid>,
) -> ApiResult<Json<BidListResponse>> {
    let bids = state.bid_service.list_by_auction(auction_id).await?;
    let total = bids.len();
    Ok(Json(BidListResponse {
        bids: bids.into_iter().map(SealedBidResponse::from).collect(),
        total,
    }))
}