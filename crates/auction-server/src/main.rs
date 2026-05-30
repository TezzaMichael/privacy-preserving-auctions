use std::sync::Arc;
use axum::Router;
use sqlx::sqlite::SqlitePoolOptions;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod auth;
mod config;
mod errors;
mod models;
mod services;
mod state;
mod storage;

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = Config::from_env();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&cfg.log_level))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&cfg.database_url)
        .await?;

    sqlx::migrate!("./src/migrations").run(&pool).await?;

    let state = Arc::new(AppState::new(pool, &cfg).await?);

    // --- TASK IN BACKGROUND PER LA CHIUSURA AUTOMATICA DELLE ASTE ---
    let bg_state = state.clone();
    tokio::spawn(async move {
        // Controlla ogni 60 secondi
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Ok(auctions) = bg_state.auction_service.list().await {
                let now = chrono::Utc::now();
                for auc in auctions {
                    
                    // 1. GESTIONE FINE OFFERTE (BiddingOpen)
                    if auc.status == auction_core::enums::AuctionStatus::BiddingOpen && auc.end_time <= now {
                        tracing::info!("Checking bids for expired auction {}", auc.id);
                        
                        let total_bids = match bg_state.bid_service.list_by_auction(auc.id).await {
                            Ok(bids) => bids.len(),
                            Err(_) => 0,
                        };

                        if total_bids == 0 {
                            tracing::info!("Asta {} scaduta con 0 offerte. Chiusura sequenziale immediata.", auc.id);
                            
                            if bg_state.auction_service.system_transition(auc.id, auction_core::enums::AuctionStatus::ClaimPhase).await.is_ok() {
                                let _ = bg_state.auction_service.system_transition(auc.id, auction_core::enums::AuctionStatus::ProofPhase).await;
                                if bg_state.auction_service.system_transition(auc.id, auction_core::enums::AuctionStatus::Closed).await.is_ok() {
                                    let payload = serde_json::json!({ 
                                        "auction_id": auc.id, 
                                        "reason": "no_bids_received" 
                                    });
                                    let _ = bg_state.bulletin_board_service.append(
                                        auc.id,
                                        auction_core::bulletin_board::EntryKind::AuctionFinalize,
                                        payload,
                                        &bg_state.server_signer
                                    ).await;
                                }
                            }
                        } else {
                            // CASO CON OFFERTE: Avvia regolarmente il Polling (ClaimPhase)
                            tracing::info!("Asta {} ha {} offerte. Avvio della ClaimPhase.", auc.id, total_bids);
                            if bg_state.auction_service.system_transition(auc.id, auction_core::enums::AuctionStatus::ClaimPhase).await.is_ok() {
                                let payload = serde_json::json!({ 
                                    "auction_id": auc.id, 
                                    "reason": "deadline_reached" 
                                });
                                let _ = bg_state.bulletin_board_service.append(
                                    auc.id,
                                    auction_core::bulletin_board::EntryKind::AuctionClose,
                                    payload,
                                    &bg_state.server_signer
                                ).await;
                            }
                        }
                    } 
                    // 2. TIMEOUT CLAIM PHASE (Indipendente dal BiddingOpen, timeout a 5 min)
                    else if auc.status == auction_core::enums::AuctionStatus::ClaimPhase {
                        if now.signed_duration_since(auc.end_time).num_minutes() >= 5 {
                            tracing::info!("ClaimPhase timeout reached for auction {} (No reveals). Closing.", auc.id);
                            
                            if bg_state.auction_service.system_transition(auc.id, auction_core::enums::AuctionStatus::Closed).await.is_ok() {
                                let payload = serde_json::json!({ 
                                    "auction_id": auc.id, 
                                    "reason": "claim_phase_timeout",
                                    "note": "Il polling è terminato ma nessun utente ha rivendicato la vittoria."
                                });
                                let _ = bg_state.bulletin_board_service.append(
                                    auc.id,
                                    auction_core::bulletin_board::EntryKind::AuctionFinalize,
                                    payload,
                                    &bg_state.server_signer
                                ).await;
                            }
                        }
                    } 
                    // 3. TIMEOUT PROOF PHASE (Indipendente, aspetta 60 minuti reali per le prove)
                    else if auc.status == auction_core::enums::AuctionStatus::ProofPhase {
                        if now.signed_duration_since(auc.end_time).num_minutes() >= 60 {
                            tracing::info!("ProofPhase timeout reached for auction {} (Missing Loser Proofs). Closing.", auc.id);
                            
                            if bg_state.auction_service.system_transition(auc.id, auction_core::enums::AuctionStatus::Closed).await.is_ok() {
                                let payload = serde_json::json!({ 
                                    "auction_id": auc.id, 
                                    "reason": "proof_phase_timeout",
                                    "note": "Il tempo per le prove di sottomissione (Loser Proofs) è scaduto."
                                });
                                let _ = bg_state.bulletin_board_service.append(
                                    auc.id,
                                    auction_core::bulletin_board::EntryKind::AuctionFinalize,
                                    payload,
                                    &bg_state.server_signer
                                ).await;
                            }
                        }
                    }
                }
            }
        }
    });
    // ----------------------------------------------------------------

    let app = Router::new()
        .merge(api::router())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("{}:{}", cfg.host, cfg.port);
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}