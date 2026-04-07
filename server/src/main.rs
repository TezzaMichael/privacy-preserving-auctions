use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use uuid::Uuid;
use serde_json::json;
use chrono::{Utc, Duration};

mod models;
mod services;

use crate::models::{
    app_state::AppState,
    enums::EntryKind,
};
use crate::services::bulletin_board::{
    append_entry, get_entries_for_auction, get_last_hash, verify_chain,
};

fn main() {
    println!("=== Avvio simulazione asta ===");

    // --- 1️⃣ Stato globale ---
    let state = AppState {
        users: Arc::new(RwLock::new(HashMap::new())),
        by_name: Arc::new(RwLock::new(HashMap::new())),
        auctions: Arc::new(RwLock::new(HashMap::new())),
        bids: Arc::new(RwLock::new(HashMap::new())),
        reveals: Arc::new(RwLock::new(HashMap::new())),
        proofs: Arc::new(RwLock::new(HashMap::new())),
        bb: Arc::new(RwLock::new(HashMap::new())),
        jwt_secret: "jwt_dummy_secret".to_string(),
        server_private_key: "server_private_key_dummy".to_string(),
        server_public_key: "server_public_key_dummy".to_string(),
        pedersen_b: "pedersen_b_dummy".to_string(),
        pedersen_b_blind: "pedersen_b_blind_dummy".to_string(),
    };

    // --- 2️⃣ Creazione utenti ---
    let user_ids: Vec<Uuid> = (1..=3).map(|_| Uuid::new_v4()).collect();
    println!("Utenti creati:");
    for (i, uid) in user_ids.iter().enumerate() {
        println!("User{} -> {}", i+1, uid);
    }

    // --- 3️⃣ Creazione asta ---
    let auction_id = Uuid::new_v4();
    let start_time = Utc::now() - Duration::seconds(10); // già iniziata
    let end_time = Utc::now() + Duration::minutes(10);

    println!("Asta creata con id: {}", auction_id);

    // --- 4️⃣ Invio bid ---
    for (i, user_id) in user_ids.iter().enumerate() {
        let commitment = format!("commitment{}", i+1);
        let entry = append_entry(
            &state,
            auction_id,
            EntryKind::SealedBid,
            json!({
                "bidder": user_id,
                "commitment": commitment,
                "timestamp": Utc::now()
            }),
        );
        println!("Bid inviato da User{}: Entry ID {}", i+1, entry.id);
    }

    // --- 5️⃣ Mostra tutte le entry dell'asta ---
    let entries = get_entries_for_auction(&state, auction_id);
    println!("\n--- Tutte le entry dell'asta ---");
    for entry in &entries {
        println!(
            "Seq {} | Kind {:?} | Bidder {:?} | Commitment {:?}",
            entry.sequence,
            entry.kind,
            entry.payload["bidder"],
            entry.payload["commitment"]
        );
    }

    // --- 6️⃣ Verifica integrità bulletin board ---
    let valid = verify_chain(&entries);
    println!("\nIntegrità hash chain: {}", valid);

    // --- 7️⃣ Ultimo hash per audit incrementale ---
    if let Some(last_hash) = get_last_hash(&state, auction_id) {
        println!("Ultimo hash dell'asta: {}", last_hash);
    }

    // --- 8️⃣ Mock selezione vincitore ---
    if !entries.is_empty() {
        let winner_entry = &entries[0]; // finto: il primo è il vincitore
        println!("\n Vincitore simulato:");
        println!(
            "Bidder: {:?}, Commitment: {:?}, Entry ID: {}",
            winner_entry.payload["bidder"],
            winner_entry.payload["commitment"],
            winner_entry.id
        );
    }

    println!("\n=== Fine simulazione asta ===");
}