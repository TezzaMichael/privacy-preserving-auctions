mod models;
mod services;

use models::bulletin_board::BulletinBoard;
use models::user::User;
use models::auction::Auction;
use models::bid::Bid;

use services::bulletin_board;

use std::time::{SystemTime, UNIX_EPOCH};

fn main() {

    println!("=== Privacy-Preserving Auction Test ===");
    
    // bulletin board
    let mut board: BulletinBoard = bulletin_board::new_board();

    // users
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

    bulletin_board::register_user(&mut board, user1);
    bulletin_board::register_user(&mut board, user2);

    println!("Users registered:");
    println!("{:?}", board.users);

    // auction
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

    println!("Auctions:");
    println!("{:?}", board.auctions);

    // bid
    let bid = Bid {
        id: 1,
        auction_id: 1,
        commitment: "fake_commitment".to_string(),
        timestamp: current_timestamp(),
    };

    bulletin_board::submit_bid(&mut board, bid);

    println!("Bids:");
    println!("{:?}", board.bids);

    println!("=== Test Completed ===");
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}