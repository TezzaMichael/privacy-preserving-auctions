// src/services/auction_service.rs

use std::collections::HashMap;

use crate::models::bulletin_board::BulletinBoard;
use crate::crypto::commitment;

/// Winner determination logic for a given auction
pub fn determine_winner(
    board: &mut BulletinBoard,
    auction_id: u64,
    openings: &HashMap<u64, (u64, String)> // bid_id -> (bid_value, nonce)
) {

    let auction = board
        .auctions
        .iter_mut()
        .find(|a| a.id == auction_id)
        .expect("Auction not found");

    println!("\n=== Winner Determination ===");

    // max --> min step by step
    let mut price = auction.max_bid;

    while price >= auction.min_bid {

        println!("Checking price: {}", price);

        for bid_id in &auction.bids {

            if let Some((bid_value, nonce)) = openings.get(bid_id) {

                if *bid_value == price {

                    // find the bid in the board
                    let bid = board.bids.iter()
                        .find(|b| b.id == *bid_id)
                        .unwrap();

                    let valid = commitment::verify(&bid.commitment, *bid_value, nonce);

                    if valid {
                        println!("Winner found! Bid ID: {} Price: {}", bid_id, price);

                        auction.winner = Some(*bid_id);
                        auction.winning_price = Some(price);
                        return;
                    } else {
                        println!("Invalid opening for bid {}", bid_id);
                    }
                }
            }
        }

        // if price is too low, stop checking
        if price < auction.step {
            break;
        }

        price -= auction.step;
    }

    println!("No valid winner found.");
}