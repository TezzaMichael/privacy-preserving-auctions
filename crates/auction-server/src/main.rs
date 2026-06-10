use std::sync::Arc;
use axum::Router;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;
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

    let db_options = SqliteConnectOptions::from_str(&cfg.database_url)?
    .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(db_options)
        .await?;

    sqlx::migrate!("./src/migrations").run(&pool).await?;

    let state = Arc::new(AppState::new(pool, &cfg).await?);

    let bg_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Ok(auctions) = bg_state.auction_service.list().await {
                let now = chrono::Utc::now();
                for auc in auctions {
                    
                    if auc.status == auction_core::enums::AuctionStatus::BiddingOpen && auc.end_time <= now {
                        tracing::info!("Checking bids for expired auction {}", auc.id);
                        
                        let total_bids = match bg_state.bid_service.list_by_auction(auc.id).await {
                            Ok(bids) => bids.len(),
                            Err(_) => 0,
                        };

                        if total_bids == 0 {
                            tracing::info!("Auction {} terminated without bids.", auc.id);
                            
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
                            tracing::info!("Auction {} has {} bids. Starting ClaimPhase.", auc.id, total_bids);
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
                    else if auc.status == auction_core::enums::AuctionStatus::ClaimPhase {
                        if now.signed_duration_since(auc.end_time).num_minutes() >= 5 {
                            tracing::info!("ClaimPhase timeout reached for auction {} (No reveals). Closing.", auc.id);
                            
                            let _ = bg_state.auction_service.system_transition(auc.id, auction_core::enums::AuctionStatus::ProofPhase).await;
                            
                            if bg_state.auction_service.system_transition(auc.id, auction_core::enums::AuctionStatus::Closed).await.is_ok() {
                                let payload = serde_json::json!({ 
                                    "auction_id": auc.id, 
                                    "reason": "claim_phase_timeout",
                                    "note": "Polling detected that no winner reveal was submitted within the expected time after bidding ended."
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
                    else if auc.status == auction_core::enums::AuctionStatus::ProofPhase {
                        if now.signed_duration_since(auc.end_time).num_minutes() >= 60 {
                            tracing::info!("ProofPhase timeout reached for auction {} (Missing Loser Proofs). Closing.", auc.id);
                            
                            if bg_state.auction_service.system_transition(auc.id, auction_core::enums::AuctionStatus::Closed).await.is_ok() {
                                let payload = serde_json::json!({ 
                                    "auction_id": auc.id, 
                                    "reason": "proof_phase_timeout",
                                    "note": "The time for submitting proofs (Loser Proofs) has expired."
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

    let app = Router::new()
        .merge(api::router())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("{}:{}", cfg.host, cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
