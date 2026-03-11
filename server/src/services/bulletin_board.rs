use crate::models::bulletin_board::BulletinBoard;
use crate::models::user::User;
use crate::models::auction::Auction;
use crate::models::bid::Bid;
use crate::models::certificate::ProofCertificate;

pub fn new_board() -> BulletinBoard {
    BulletinBoard {
        users: Vec::new(),
        auctions: Vec::new(),
        bids: Vec::new(),
        certificates: Vec::new(),
    }
}

pub fn register_user(board: &mut BulletinBoard, user: User) {
    board.users.push(user);
}

pub fn create_auction(board: &mut BulletinBoard, auction: Auction) {
    board.auctions.push(auction);
}

pub fn submit_bid(board: &mut BulletinBoard, bid: Bid) {
    let bid_id = bid.id;
    let auction_id = bid.auction_id;

    board.bids.push(bid);

    if let Some(auction) = board.auctions.iter_mut().find(|a| a.id == auction_id) {
        auction.bids.push(bid_id);
    }
}

pub fn publish_certificate(board: &mut BulletinBoard, certificate: ProofCertificate) {
    board.certificates.push(certificate);
}

pub fn get_auction(board: &BulletinBoard, auction_id: u64) -> Option<&Auction> {
    board.auctions.iter().find(|a| a.id == auction_id)
}