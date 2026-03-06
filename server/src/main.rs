mod models;

use models::user::User;
use models::auction::Auction;
use models::bid::Bid;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    println!("=== Privacy-Preserving Auction Simulation ===");

    // 1️⃣ Crea utenti di esempio
    let user1 = User {
        id: 1,
        username: "alice".to_string(),
        password_hash: "hash1".to_string(),
    };

    let user2 = User {
        id: 2,
        username: "bob".to_string(),
        password_hash: "hash2".to_string(),
    };

    println!("Registered users:");
    println!("{:?}", user1);
    println!("{:?}", user2);

    // 2️⃣ Crea un'asta di esempio
    let auction = Auction {
        id: 1,
        min_bid: 1000,
        max_bid: 2000,
        step: 100,
        start_time: current_timestamp(),
        end_time: current_timestamp() + 3600, // durata 1h
        bids: vec![],
        winner: None,
        winning_price: None,
    };

    println!("\nCreated auction:");
    println!("{:?}", auction);

    // 3️⃣ Simula bid degli utenti
    let bid1 = Bid {
        id: 1,
        auction_id: auction.id,
        user_id: user1.id,
        commitment: "commitment1".to_string(), // qui in futuro hash(bid || nonce)
        timestamp: current_timestamp(),
    };

    let bid2 = Bid {
        id: 2,
        auction_id: auction.id,
        user_id: user2.id,
        commitment: "commitment2".to_string(),
        timestamp: current_timestamp(),
    };

    println!("\nSubmitted bids:");
    println!("{:?}", bid1);
    println!("{:?}", bid2);

    println!("\nSimulation complete.");
}

// Funzione helper per timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}