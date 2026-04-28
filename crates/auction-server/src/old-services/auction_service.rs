use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::models::{
    app_state::AppState,
    auction::Auction,
    enums::EntryKind,
};

use crate::services::bulletin_board;



/// Crea una nuova asta
///
/// Azioni:
/// 1) valida parametri
/// 2) salva in AppState
/// 3) scrive entry AuctionCreate sul Bulletin Board
pub fn create_auction(
    state: &AppState,
    name: String,
    description: String,
    min_bid: u64,
    max_bid: u64,
    step: u64,
    start_time: chrono::DateTime<Utc>,
    end_time: chrono::DateTime<Utc>,
    claim_secs: u64,
    proof_secs: u64,
    created_by: Uuid,
) -> Result<Auction, String> {

    if min_bid >= max_bid {
        return Err("min_bid deve essere < max_bid".into());
    }

    if step == 0 {
        return Err("step non può essere zero".into());
    }

    if start_time >= end_time {
        return Err("start_time deve essere < end_time".into());
    }

    let auction = Auction {

        id: Uuid::new_v4(),

        name,

        description,

        min_bid,

        max_bid,

        step,

        start_time,

        end_time,

        claim_secs,

        proof_secs,

        winner_id: None,

        winning_price: None,

        winner_commitment: None,

        winner_blinding: None,

        created_by,

        created_at: Utc::now(),
    };



    {
        let mut auctions = state.auctions.write().unwrap();

        auctions.insert(auction.id, auction.clone());
    }



    // scrittura su Bulletin Board (zero-trust transcript)
    bulletin_board::append_entry(
        state,
        auction.id,
        EntryKind::AuctionCreate,
        json!(auction),
    );



    Ok(auction)
}



/// Recupera un'asta
pub fn get_auction(
    state: &AppState,
    auction_id: Uuid,
) -> Option<Auction> {

    let auctions = state.auctions.read().unwrap();

    auctions.get(&auction_id).cloned()
}



/// Aggiorna risultato asta dopo reveal vincitore
///
/// chiamato da reveal_service
pub fn set_winner(
    state: &AppState,
    auction_id: Uuid,
    winner_id: Uuid,
    winning_price: u64,
    winner_commitment: String,
    winner_blinding: String,
) -> Result<(), String> {

    let mut auctions = state.auctions.write().unwrap();

    let auction = auctions
        .get_mut(&auction_id)
        .ok_or("asta non trovata")?;



    auction.winner_id = Some(winner_id);

    auction.winning_price = Some(winning_price);

    auction.winner_commitment = Some(winner_commitment);

    auction.winner_blinding = Some(winner_blinding);



    Ok(())
}



/// Chiude asta (entry finale transcript)
///
/// chiamato dopo verifica proof losers
pub fn close_auction(
    state: &AppState,
    auction_id: Uuid,
) -> Result<(), String> {

    {
        let auctions = state.auctions.read().unwrap();

        if !auctions.contains_key(&auction_id) {
            return Err("asta non trovata".into());
        }
    }



    bulletin_board::append_entry(
        state,
        auction_id,
        EntryKind::AuctionClose,
        json!({
            "auction_id": auction_id,
            "closed_at": Utc::now()
        }),
    );



    Ok(())
}



/// Conta numero offerte ricevute
///
/// utile per AuctionResponse DTO
pub fn count_bids(
    state: &AppState,
    auction_id: Uuid,
) -> usize {

    let bids = state.bids.read().unwrap();

    bids
        .values()
        .filter(|b| b.auction_id == auction_id)
        .count()
}