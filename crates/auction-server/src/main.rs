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

    // In privacy-preserving-auctions/crates/auction-server/src/main.rs

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
                    
                    // 1. Timeout di fine asta: Passaggio da BiddingOpen a ClaimPhase
                    if auc.status == auction_core::enums::AuctionStatus::BiddingOpen && auc.end_time <= now {
                        tracing::info!("Auto-closing auction {}", auc.id);
                        
                        // Passa alla fase di Claim
                        if bg_state.auction_service.system_transition(auc.id, auction_core::enums::AuctionStatus::ClaimPhase).await.is_ok() {
                            // Registra la chiusura sulla Bulletin Board
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
                    // --- NUOVO BLOCCO: Da ClaimPhase a ProofPhase ---
                    // Impostato a 5 minuti dopo la fine dell'asta per comodità di demo.
                    // Puoi cambiare `num_minutes() >= 5` in `num_hours() >= 24` per la produzione.
                    else if auc.status == auction_core::enums::AuctionStatus::ClaimPhase 
                         && now.signed_duration_since(auc.end_time).num_minutes() >= 5
                    {
                        tracing::info!("ClaimPhase timer ended. Moving auction {} to ProofPhase", auc.id);
                        
                        // Forza il passaggio alla generazione delle prove
                        let _ = bg_state.auction_service.system_transition(auc.id, auction_core::enums::AuctionStatus::ProofPhase).await;
                    }
                    // ------------------------------------------------
                    // 2. Timeout della Proof Phase: Passaggio da ProofPhase a Closed
                    else if auc.status == auction_core::enums::AuctionStatus::ProofPhase 
                         && now.signed_duration_since(auc.end_time).num_hours() >= 24 
                    {
                        tracing::info!("ProofPhase timeout reached for auction {}", auc.id);
                        
                        // Forza la chiusura dell'asta
                        if bg_state.auction_service.system_transition(auc.id, auction_core::enums::AuctionStatus::Closed).await.is_ok() {
                            let payload = serde_json::json!({ 
                                "auction_id": auc.id, 
                                "reason": "proof_phase_timeout",
                                "note": "Timeout raggiunto: non tutti i perdenti hanno inviato le prove."
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