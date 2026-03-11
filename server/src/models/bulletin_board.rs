use crate::models::user::User;
use crate::models::auction::Auction;
use crate::models::bid::Bid;
use crate::models::certificate::ProofCertificate;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulletinBoard {
    pub users: Vec<User>,
    pub auctions: Vec<Auction>,
    pub bids: Vec<Bid>,
    pub certificates: Vec<ProofCertificate>,
}