use crate::models::user::User;
use crate::models::auction::Auction;
use crate::models::bid::Bid;
use crate::models::certificate::ProofCertificate;

pub struct BulletinBoard {
    pub users: Vec<User>,
    pub auctions: Vec<Auction>,
    pub bids: Vec<Bid>,
    pub certificates: Vec<ProofCertificate>,
}

impl BulletinBoard {
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            auctions: Vec::new(),
            bids: Vec::new(),
            certificates: Vec::new(),
        }
    }

    pub fn register_user(&mut self, user: User) {
        self.users.push(user);
    }

    pub fn create_auction(&mut self, auction: Auction) {
        self.auctions.push(auction);
    }

    pub fn submit_bid(&mut self, bid: Bid) {
        self.bids.push(bid);
    }

    pub fn publish_certificate(&mut self, certificate: ProofCertificate) {
        self.certificates.push(certificate);
    }
}