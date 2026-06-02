use std::sync::Arc;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{post},
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
        .route("/auctions/:id/reveal", post(reveal_winner).get(get_winner_reveal))
        .route("/auctions/:id/loser-proofs", post(submit_loser_proof).get(list_loser_proofs))
}

async fn reveal_winner(
    State(state): State<Arc<AppState>>,
    AuthUser(user_id): AuthUser,
    Path(auction_id): Path<Uuid>,
    Json(req): Json<RevealWinnerRequest>,
) -> ApiResult<Json<RevealWinnerResponse>> {
    
    let auction = state.auction_service.get(auction_id).await?;
    if auction.status != AuctionStatus::ClaimPhase && auction.status != AuctionStatus::ProofPhase {
        return Err(AuctionError::Internal("Auction is not in claim or proof phase.".into()).into());
    }

    let current_winner_opt = state.proof_service.get_winner_reveal(auction_id).await?;
    let revealed_val = req.revealed_value as i64;

    if current_winner_opt.is_none() {
        let now = chrono::Utc::now();
        let claim_start_time = auction.end_time;
        let max_bid = auction.max_bid.unwrap_or(0) as i64;
        let bid_step = auction.bid_step as i64;
        let seconds_per_step = 2; 
        let warm_up_seconds = 60; 
        let elapsed_seconds = now.signed_duration_since(claim_start_time).num_seconds().max(0);
        
        let current_polling_price = if elapsed_seconds < warm_up_seconds {
            max_bid
        } else {
            let elapsed_after_warmup = elapsed_seconds - warm_up_seconds;
            let steps_passed = elapsed_after_warmup / seconds_per_step;
            max_bid - (steps_passed * bid_step)
        };
        
        let upper_bound = current_polling_price + bid_step;
        let lower_bound = current_polling_price - bid_step;

        if revealed_val > upper_bound || revealed_val < lower_bound {
            return Err(AuctionError::Internal(
                format!(
                    "Out of sync request! Current polling is at {}. You declared {}.",
                    current_polling_price, req.revealed_value
                ).into()
            ).into());
        }
    }

    let bid = state.bid_service.get_by_id(req.bid_id).await?;
    if bid.auction_id != auction_id {
        return Err(AuctionError::BidderNotInAuction(user_id, auction_id).into());
    }
    if bid.bidder_id != user_id {
        return Err(AuctionError::Internal("Unauthorized action.".into()).into());
    }

    let mut retries = 0;
    
    let entry = loop {
        let current_winner_opt = state.proof_service.get_winner_reveal(auction_id).await?;
        let mut is_winner = true;
        let mut oust_current = false;

        if let Some(ref current_winner) = current_winner_opt {
            let old_bid = state.bid_service.get_by_id(current_winner.bid_id).await?;
            let old_seq = old_bid.bb_sequence.unwrap_or(i64::MAX);
            let new_seq = bid.bb_sequence.unwrap_or(i64::MAX);

            if req.revealed_value < current_winner.revealed_value as u64 {
                is_winner = false;
            } else if req.revealed_value == current_winner.revealed_value as u64 {
                if new_seq < old_seq {
                    oust_current = true;
                } else {
                    is_winner = false;
                }
            } else {
                oust_current = true;
            }
        }

        if !is_winner {
            let winner_val = current_winner_opt.as_ref().unwrap().revealed_value;
            let record = state.proof_service.submit_loser_proof(
                auction_id, user_id, bid.id, req.proof_json.clone(), &bid.commitment_hex, winner_val,
            ).await?;

            let payload = serde_json::to_value(LoserProofPayload {
                proof_id: record.id, auction_id, bidder_id: user_id, bid_id: bid.id,
            })?;
            let entry = state.bulletin_board_service.append(
                auction_id, auction_core::bulletin_board::EntryKind::LoserProof, payload, &state.server_signer
            ).await?;
            state.proof_service.update_loser_bb_sequence(record.id, entry.sequence).await?;
            
            check_auto_close(&state, auction_id).await;
            
            return Ok(Json(RevealWinnerResponse {
                reveal_id: record.id, winner_id: user_id, revealed_value: req.revealed_value as i64,
                bb_entry_hash_hex: entry.entry_hash_hex, bb_sequence: Some(entry.sequence),
            }));
        }

        if oust_current {
            let old_winner = current_winner_opt.unwrap();
            let old_bid = state.bid_service.get_by_id(old_winner.bid_id).await?;
            
            let record = match state.proof_service.override_winner_reveal(
                old_winner.id, auction_id, user_id, bid.id, req.revealed_value,
                req.proof_json.clone(), &bid.commitment_hex, &state.pedersen_generators,
            ).await {
                Ok(r) => r,
                Err(e) => {
                    if matches!(e, AuctionError::InvalidProof | AuctionError::InvalidCommitment) {
                        return Err(e.into());
                    }
                    retries += 1;
                    if retries > 3 { return Err(e.into()); }
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                    continue;
                }
            };

            let loser_record = state.proof_service.submit_loser_proof(
                auction_id, old_winner.winner_id, old_winner.bid_id, old_winner.proof_json.clone(),
                &old_bid.commitment_hex, req.revealed_value as i64,
            ).await?;

            let loser_payload = serde_json::to_value(LoserProofPayload {
                proof_id: loser_record.id, auction_id, bidder_id: old_winner.winner_id, bid_id: old_winner.bid_id,
            })?;
            let _ = state.bulletin_board_service.append(
                auction_id, auction_core::bulletin_board::EntryKind::LoserProof, loser_payload, &state.server_signer
            ).await?;
            
            let payload = serde_json::to_value(WinnerRevealPayload {
                reveal_id: record.id, auction_id, winner_id: user_id, bid_id: bid.id,
                revealed_value: record.revealed_value, proof_json: req.proof_json,
            })?;
            let bb_entry = state.bulletin_board_service.append(
                auction_id, auction_core::bulletin_board::EntryKind::WinnerReveal, payload, &state.server_signer
            ).await?;
            state.proof_service.update_winner_bb_sequence(record.id, bb_entry.sequence).await?;
            
            check_auto_close(&state, auction_id).await;

            return Ok(Json(RevealWinnerResponse {
                reveal_id: record.id, winner_id: user_id, revealed_value: record.revealed_value,
                bb_entry_hash_hex: bb_entry.entry_hash_hex, bb_sequence: Some(bb_entry.sequence),
            }));
        }

        let record = match state.proof_service.submit_winner_reveal(
            auction_id, user_id, bid.id, req.revealed_value, req.proof_json.clone(), &bid.commitment_hex, &state.pedersen_generators,
        ).await {
            Ok(r) => r,
            Err(e) => {
                if matches!(e, AuctionError::InvalidProof | AuctionError::InvalidCommitment) {
                    return Err(e.into());
                }
                retries += 1;
                if retries > 3 { return Err(e.into()); }
                tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                continue;
            }
        };

        let payload = serde_json::to_value(WinnerRevealPayload {
            reveal_id: record.id, auction_id, winner_id: user_id, bid_id: bid.id,
            revealed_value: record.revealed_value, proof_json: req.proof_json,
        })?;
        let bb_entry = state.bulletin_board_service.append(
            auction_id, auction_core::bulletin_board::EntryKind::WinnerReveal, payload, &state.server_signer
        ).await?;
        state.proof_service.update_winner_bb_sequence(record.id, bb_entry.sequence).await?;

        break (record, bb_entry);
    };

    if let Ok(all_bids) = state.bid_service.list_by_auction(auction_id).await {
        if all_bids.len() <= 1 {
            let _ = state.auction_service.system_transition(auction_id, AuctionStatus::ProofPhase).await;
            if state.auction_service.system_transition(auction_id, AuctionStatus::Closed).await.is_ok() {
                let final_payload = serde_json::json!({
                    "auction_id": auction_id, "verified_winner_id": user_id, "total_bids": all_bids.len()
                });
                let _ = state.bulletin_board_service.append(
                    auction_id, auction_core::bulletin_board::EntryKind::AuctionFinalize, final_payload, &state.server_signer
                ).await;
            }
        } else {
            let _ = state.auction_service.system_transition(auction_id, AuctionStatus::ProofPhase).await;
        }
    }

    Ok(Json(RevealWinnerResponse {
        reveal_id: entry.0.id, winner_id: user_id, revealed_value: entry.0.revealed_value,
        bb_entry_hash_hex: entry.1.entry_hash_hex, bb_sequence: Some(entry.1.sequence),
    }))
}

async fn get_winner_reveal(
    State(state): State<Arc<AppState>>,
    Path(auction_id): Path<Uuid>,
) -> impl axum::response::IntoResponse {
    match state.proof_service.get_winner_reveal(auction_id).await {
        Ok(Some(record)) => axum::response::Json(WinnerRevealDetailResponse::from(record)).into_response(),
        Ok(None) => axum::response::Json(serde_json::Value::Null).into_response(),
        Err(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
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
        .ok_or_else(|| AuctionError::Internal("No winner reveal found.".into()))?;

    if winner.winner_id == user_id {
        return Err(AuctionError::Internal("The verified winner cannot submit a loser proof.".into()).into());
    }

    let verification_result = auction_verifier::loser::verify_loser_proof(
        &bid.commitment_hex,
        &req.proof_json,
        winner.revealed_value as u64,
    );

    match verification_result {
        Ok(()) => {}
        Err(auction_verifier::loser::LoserVerifyError::BidGreaterThanWinner) => {
            state.auction_service.system_transition(auction_id, AuctionStatus::ClaimPhase).await?;
            state.proof_service.delete_winner(auction_id).await?;
            state.proof_service.delete_all_loser_proofs(auction_id).await?;
            
            let payload = serde_json::json!({
                "auction_id": auction_id,
                "challenger_id": user_id,
                "reason": "valid_higher_bid_discovered"
            });
            let _ = state.bulletin_board_service.append(
                auction_id,
                auction_core::bulletin_board::EntryKind::AuctionReverted,
                payload,
                &state.server_signer
            ).await?;

            return Err(AuctionError::Internal(
                "Challenge successful. The current winner was invalidated and the auction has returned to the Claim Phase.".into()
            ).into());
        }
        Err(_) => {
            return Err(AuctionError::InvalidProof.into());
        }
    }

    let record = state.proof_service.submit_loser_proof(
        auction_id,
        user_id,
        bid.id,
        req.proof_json.clone(),
        &bid.commitment_hex,
        winner.revealed_value,
    ).await?;
    
    check_auto_close(&state, auction_id).await;

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

async fn check_auto_close(state: &Arc<AppState>, auction_id: Uuid) {
    if let Ok(all_bids) = state.bid_service.list_by_auction(auction_id).await {
        if all_bids.len() <= 1 {
            let _ = state.auction_service.system_transition(auction_id, AuctionStatus::ProofPhase).await;
            if state.auction_service.system_transition(auction_id, AuctionStatus::Closed).await.is_ok() {
                let final_payload = serde_json::json!({
                    "auction_id": auction_id, "reason": "single_bidder_auto_close"
                });
                let _ = state.bulletin_board_service.append(
                    auction_id, auction_core::bulletin_board::EntryKind::AuctionFinalize, final_payload, &state.server_signer
                ).await;
            }
        } else if let Ok(all_proofs) = state.proof_service.get_loser_proofs(auction_id).await {
            let winner_exists = state.proof_service.get_winner_reveal(auction_id).await.unwrap_or(None).is_some();
            let total_proofs = all_proofs.len() + if winner_exists { 1 } else { 0 };
            
            if total_proofs >= all_bids.len() {
                let _ = state.auction_service.system_transition(auction_id, AuctionStatus::ProofPhase).await;
                if state.auction_service.system_transition(auction_id, AuctionStatus::Closed).await.is_ok() {
                    let final_payload = serde_json::json!({
                        "auction_id": auction_id, "reason": "all_proofs_submitted"
                    });
                    let _ = state.bulletin_board_service.append(
                        auction_id, auction_core::bulletin_board::EntryKind::AuctionFinalize, final_payload, &state.server_signer
                    ).await;
                }
            }
        }
    }
}