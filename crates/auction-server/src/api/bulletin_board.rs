use std::sync::Arc;
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use uuid::Uuid;
use auction_core::responses::{BulletinBoardEntryResponse, BulletinBoardResponse};
use crate::{errors::ApiResult, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/bulletin-board/:auction_id",           get(get_chain))
        .route("/bulletin-board/:auction_id/:sequence", get(get_entry))
}

async fn get_chain(
    State(state): State<Arc<AppState>>,
    Path(auction_id): Path<Uuid>,
) -> ApiResult<Json<BulletinBoardResponse>> {
    let entries = state.bulletin_board_service.get_chain(auction_id).await?;
    let head_hash_hex = entries.last().map(|e| e.entry_hash_hex.clone()).unwrap_or_else(|| "0".repeat(64));
    let total = entries.len();
    Ok(Json(BulletinBoardResponse { auction_id, entries, total, head_hash_hex }))
}

async fn get_entry(
    State(state): State<Arc<AppState>>,
    Path((auction_id, sequence)): Path<(Uuid, i64)>,
) -> ApiResult<Json<BulletinBoardEntryResponse>> {
    let entry = state.bulletin_board_service.get_entry(auction_id, sequence).await?
        .ok_or_else(|| auction_core::errors::AuctionError::Internal("entry not found".into()))?;
    Ok(Json(BulletinBoardEntryResponse { entry }))
}