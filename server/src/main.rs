mod models;
mod services;
mod api;
mod crypto;

use axum::http::response;
use models::bulletin_board::BulletinBoard;
use models::user::User;
use models::auction::Auction;
use models::bid::Bid;
use models::certificate::ProofCertificate;

use services::bulletin_board;
use services::auction_service;

use crate::crypto::commitment;

use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

fn main() {

    println!("=== Privacy-Preserving Auction Test ===");

    // -------------------------
    // 1. Bulletin Board
    // -------------------------
    let mut board: BulletinBoard = bulletin_board::new_board();

    // -------------------------
    // 2. Users
    // -------------------------
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


    // -------------------------
    // 3. Auction
    // -------------------------
    let auction = Auction {
        id: 1,
        min_bid: 1000,
        max_bid: 2000,
        step: 100,
        start_time: current_timestamp(),
        end_time: current_timestamp() + 3600,
        bids: vec![],
        winner: None,
        winning_price: None,
    };

    bulletin_board::create_auction(&mut board, auction);

    println!("\nAuction created:");
    println!("{:#?}", board.auctions);

    // -------------------------
    // 4. Bids (PRIVATE)
    // -------------------------

    // Alice bid = 1500
    let alice_bid_value = 1500;
    let alice_nonce = commitment::generate_nonce();
    let alice_commitment = commitment::commit(alice_bid_value, &alice_nonce);

    let bid1 = Bid {
        id: 1,
        user_id: 1,
        auction_id: 1,
        commitment: alice_commitment.clone(),
        timestamp: current_timestamp(),
    };

    // Bob bid = 1700
    let bob_bid_value = 1700;
    let bob_nonce = commitment::generate_nonce();
    let bob_commitment = commitment::commit(bob_bid_value, &bob_nonce);

    let bid2 = Bid {
        id: 2,
        user_id: 2,
        auction_id: 1,
        commitment: bob_commitment.clone(),
        timestamp: current_timestamp(),
    };
    
    // Creation certificates (simulated)
    let cert1 = crate::models::certificate::ProofCertificate {
        bidder_id: 1,
        auction_id: 1,
        challenge: "challenge".to_string(),
        response: "response1".to_string(),
    };

    let cert2 = crate::models::certificate::ProofCertificate {
        bidder_id: 2,
        auction_id: 1,
        challenge: "challenge".to_string(),
        response: "cert".to_string(),
    };

    bulletin_board::submit_bid(&mut board, bid1);
    bulletin_board::submit_bid(&mut board, bid2);
    bulletin_board::publish_certificate(&mut board, cert1);
    bulletin_board::publish_certificate(&mut board, cert2);

    // -------------------------
    // 🔐 PUBLIC BULLETIN BOARD
    // -------------------------
    println!("\n=== Bulletin Board (Public View) ===");

    for bid in &board.bids {
        println!(
            "timestamp: {} | commitment: {} | certificates: {}",
            bid.timestamp,
            bid.commitment,
            board.certificates.iter()
                .filter(|c| c.auction_id == bid.auction_id && c.bidder_id == bid.user_id)
                .map(|c| format!("{}", c.response))
                .collect::<Vec<String>>()
                .join(", ")
        );
    }
    // -------------------------
    // 5. Opening simulation
    // -------------------------
    println!("\n=== Opening Phase (Simulation) ===");

    println!("Bob reveals bid + nonce");

    let is_valid = commitment::verify(&bob_commitment, bob_bid_value, &bob_nonce);

    println!("Verification result: {}", is_valid);

    if is_valid {
        println!("Bob's bid is VALID and can be accepted");
    } else {
        println!("Invalid opening!");
    }

    // -------------------------
    // 6. Winner determination
    // -------------------------
    let mut openings = HashMap::new();

    openings.insert(1, (alice_bid_value, alice_nonce.clone()));
    openings.insert(2, (bob_bid_value, bob_nonce.clone()));

    auction_service::determine_winner(&mut board, 1, &openings);

    println!("\n=== Auction Result ===");
    println!("{:#?}", board.auctions);

    // -------------------------
    // DEBUG (NOT PUBLIC)
    // -------------------------
    println!("\n[DEBUG - SECRET VALUES]");
    println!("Alice bid: {}", alice_bid_value);
    println!("Bob bid: {}", bob_bid_value);

    println!("\n=== Test Completed ===");
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}