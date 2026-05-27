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
use chrono::Utc;

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

    let auction = state.auction_service.get(auction_id).await?;
    let now = chrono::Utc::now();
    let claim_start_time = auction.end_time;
    
    let max_bid = auction.max_bid.unwrap_or(0) as i64; 
    let bid_step = auction.bid_step as i64;
    let seconds_per_step = 10;
    
    // REINTRODOTTI I 2 MINUTI DI ATTESA
    let warm_up_seconds = 120; 
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
    let revealed_val = req.revealed_value as i64;

    if revealed_val > upper_bound || revealed_val < lower_bound {
        return Err(AuctionError::Internal(
            format!(
                "Richiesta fuori sincrono! Il polling attuale è a {}. Tu hai dichiarato {}.", 
                current_polling_price, req.revealed_value
            ).into()
        ).into());
    }

    let bid = state.bid_service.get_by_id(req.bid_id).await?;
    if bid.auction_id != auction_id {
        return Err(AuctionError::BidderNotInAuction(user_id, auction_id).into());
    }
    if bid.bidder_id != user_id {
        return Err(AuctionError::NotCreator.into());
    }

    let current_winner_opt = state.proof_service.get_winner_reveal(auction_id).await?;
    let record = if let Some(current_winner) = current_winner_opt {
        if req.revealed_value <= current_winner.revealed_value as u64 {
             return Err(AuctionError::Internal("Un'offerta più alta è già stata rivelata come vincitrice provvisoria.".into()).into());
        }
        state.proof_service.override_winner_reveal(
            current_winner.id, auction_id, user_id, bid.id, req.revealed_value,
            req.proof_json.clone(), &bid.commitment_hex, &state.pedersen_generators,
        ).await?
    } else {
        state.proof_service.submit_winner_reveal(
            auction_id, user_id, bid.id, req.revealed_value,
            req.proof_json.clone(), &bid.commitment_hex, &state.pedersen_generators,
        ).await?
    };

    let payload = serde_json::to_value(WinnerRevealPayload {
        reveal_id: record.id, auction_id, winner_id: user_id, bid_id: bid.id,
        revealed_value: record.revealed_value, proof_json: req.proof_json,
    })?;
    
    let entry = state.bulletin_board_service
        .append(auction_id, auction_core::bulletin_board::EntryKind::WinnerReveal, payload, &state.server_signer)
        .await?;
        
    state.proof_service.update_winner_bb_sequence(record.id, entry.sequence).await?;

    // TRANSIZIONE ISTANTANEA ALLA FASE SUCCESSIVA (Senza attendere il cron)
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
        reveal_id: record.id, winner_id: user_id, revealed_value: record.revealed_value,
        bb_entry_hash_hex: entry.entry_hash_hex, bb_sequence: Some(entry.sequence),
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
        req.proof_json.clone(),
        &bid.commitment_hex,
        winner.revealed_value,
    ).await?;

    let payload = serde_json::to_value(LoserProofPayload {
        proof_id: record.id,
        auction_id,
        bidder_id: user_id,
        bid_id: bid.id,
    })?;
    
    let _entry = state.bulletin_board_service
        .append(auction_id, auction_core::bulletin_board::EntryKind::LoserProof, payload, &state.server_signer)
        .await?;

    // OTTIMIZZAZIONE: Se tutti i perdenti hanno inviato la prova, chiudiamo l'asta subito
    if let Ok(all_bids) = state.bid_service.list_by_auction(auction_id).await {
        if let Ok(all_proofs) = state.proof_service.get_loser_proofs(auction_id).await {
            // Se le prove inviate coprono tutti i bid (tranne 1, che è il vincitore)
            if all_proofs.len() >= all_bids.len().saturating_sub(1) {
                tracing::info!("Tutti i perdenti hanno inviato la prova per {}. Chiusura asta immediata.", auction_id);
                
                if state.auction_service.system_transition(auction_id, AuctionStatus::Closed).await.is_ok() {
                    let final_payload = serde_json::json!({ 
                        "auction_id": auction_id, 
                        "reason": "all_loser_proofs_submitted" 
                    });
                    let _ = state.bulletin_board_service.append(
                        auction_id,
                        auction_core::bulletin_board::EntryKind::AuctionFinalize,
                        final_payload,
                        &state.server_signer
                    ).await;
                }
            }
        }
    }

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
