mod models;
mod bulletin_board;

use models::user::User;
use models::auction::Auction;
use models::bid::Bid;

use bulletin_board::board::BulletinBoard;

use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    println!("=== Privacy-Preserving Auction Simulation ===");

    let mut board = BulletinBoard::new();

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

    board.register_user(user1.clone());
    board.register_user(user2.clone());

    println!("Registered users:");
    println!("{:?}", user1);
    println!("{:?}", user2);

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

    board.create_auction(auction.clone());

    println!("\nCreated auction:");
    println!("{:?}", auction);

    let bid1 = Bid {
        id: 1,
        auction_id: auction.id,
        user_id: user1.id,
        commitment: "commitment1".to_string(),
        timestamp: current_timestamp(),
    };

    let bid2 = Bid {
        id: 2,
        auction_id: auction.id,
        user_id: user2.id,
        commitment: "commitment2".to_string(),
        timestamp: current_timestamp(),
    };

    board.submit_bid(bid1.clone());
    board.submit_bid(bid2.clone());

    println!("\nSubmitted bids:");
    println!("{:?}", bid1);
    println!("{:?}", bid2);

    println!("\nBulletin Board state:");
    println!("Users: {:?}", board.users);
    println!("Auctions: {:?}", board.auctions);
    println!("Bids: {:?}", board.bids);

    println!("\nSimulation complete.");
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}