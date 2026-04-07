use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::models::{
    app_state::AppState,
    bid::Bid,
    enums::{EntryKind, BidStatus},
};

use crate::services::bulletin_board;
use crate::services::auction_service;

/// Submit sealed commitment bid
///
/// Il client invia:
/// commitment = H(value || nonce)
///
/// Il server NON conosce value.
///
/// Scrive:
/// EntryKind::SealedBid nel bulletin board
pub fn submit_bid(
    state: &AppState,
    auction_id: Uuid,
    user_id: Uuid,
    commitment: String,
    user_signature: String,
) -> Result<Bid, String> {
    // Recupera asta
    let auction = auction_service::get_auction(state, auction_id)
        .ok_or("Asta non trovata")?;

    let now = Utc::now();

    // Verifica finestra temporale bidding
    if now < auction.start_time {
        return Err("Asta non ancora iniziata".into());
    }
    if now > auction.end_time {
        return Err("Asta terminata".into());
    }

    // Crea bid sealed
    let bid = Bid {
        id: Uuid::new_v4(),
        auction_id,
        user_id,
        commitment: commitment.clone(),
        submitted_at: now,
        signature: user_signature.clone(),
        status: BidStatus::Sealed,
    };

    // Salva bid nello stato globale
    {
        let mut bids = state.bids.write().unwrap();
        bids.insert((bid.auction_id, bid.user_id), bid.clone());
    }

    // Scrive transcript verificabile nel Bulletin Board
    bulletin_board::append_entry(
        state,
        auction_id,
        EntryKind::SealedBid,
        json!({
            "bid_id": bid.id,
            "user_id": user_id,
            "commitment": commitment,
            "timestamp": bid.submitted_at,
            "signature": user_signature,
        }),
    );

    Ok(bid)
}