use sqlx::SqlitePool;
use uuid::Uuid;
use auction_core::{auction::Auction, enums::AuctionStatus, errors::AuctionError};
use crate::storage::auction_repo::AuctionRepo;

pub struct AuctionService {
    repo: AuctionRepo,
}

impl AuctionService {
    pub fn new(pool: SqlitePool) -> Self { Self { repo: AuctionRepo(pool) } }

    pub async fn create(
        &self,
        creator_id: Uuid,
        title: String,
        description: String,
        min_bid: i64,
        max_bid: i64, // <-- ORA È OBBLIGATORIO (i64 puro)
        bid_step: i64,
        duration_seconds: i64,
    ) -> Result<Auction, AuctionError> {
        
        if bid_step <= 0 {
            return Err(AuctionError::Internal("Il bid_step deve essere rigorosamente maggiore di zero.".into()));
        }

        // Validazione di coerenza diretta (senza Option)
        if min_bid > max_bid {
            return Err(AuctionError::Internal("Il min_bid non può essere superiore al max_bid.".into()));
        }
        if bid_step > max_bid {
            return Err(AuctionError::Internal("Il bid_step non può essere superiore al max_bid.".into()));
        }
        // Controllo opzionale ma consigliato: il salto non dovrebbe essere più grande del range giocabile
        if bid_step > (max_bid - min_bid) && min_bid != max_bid {
            return Err(AuctionError::Internal("Il bid_step è troppo grande rispetto al range tra min_bid e max_bid.".into()));
        }

        // NOTA: Se la tua struct in `auction_core` usa ancora Option<i64> per il database,
        // avvolgiamo max_bid in Some(). Se hai cambiato anche lì in i64 puro, togli Some().
        let mut auction = Auction::new(creator_id, title, description, min_bid, Some(max_bid), bid_step, duration_seconds);
        
        // AUTO-AVVIO: Sovrascriviamo lo stato iniziale (solitamente Pending) per farla partire subito
        auction.status = AuctionStatus::BiddingOpen;

        self.repo.insert(&auction).await?;
        Ok(auction)
    }

    pub async fn get(&self, id: Uuid) -> Result<Auction, AuctionError> {
        self.repo.find_by_id(id).await
    }

    pub async fn list(&self) -> Result<Vec<Auction>, AuctionError> {
        self.repo.list_all().await
    }

    pub async fn system_transition(
        &self,
        id: Uuid,
        to: AuctionStatus,
    ) -> Result<Auction, AuctionError> {
        let auction = self.repo.find_by_id(id).await?;
        if !auction.status.can_transition_to(&to) {
            return Err(AuctionError::InvalidStateTransition {
                from: auction.status,
                to,
            });
        }
        self.repo.update_status(id, &to).await?;
        self.repo.find_by_id(id).await
    }

    pub async fn transition(
    &self,
    id: Uuid,
    requester_id: Uuid,
    to: AuctionStatus,
) -> Result<Auction, AuctionError> {
    let auction = self.repo.find_by_id(id).await?;

    if auction.creator_id != requester_id {
        return Err(AuctionError::NotCreator);
    }

    // Aggiungi il time-lock per impedire la chiusura prematura
    if to == AuctionStatus::ClaimPhase && chrono::Utc::now() < auction.end_time {
        return Err(AuctionError::Internal("L'asta non può essere chiusa prima della scadenza".into()));
    }

    if !auction.status.can_transition_to(&to) {
        return Err(AuctionError::InvalidStateTransition {
            from: auction.status,
            to,
        });
    }

    self.repo.update_status(id, &to).await?;
    self.repo.find_by_id(id).await
}

    pub async fn set_bb_sequence(&self, id: Uuid, seq: i64) -> Result<(), AuctionError> {
        self.repo.update_bb_sequence(id, seq).await
    }

    pub async fn set_server_signature(&self, id: Uuid, sig_hex: &str) -> Result<(), AuctionError> {
        self.repo.update_server_signature(id, sig_hex).await
    }

    pub async fn require_status(&self, id: Uuid, required: &AuctionStatus) -> Result<Auction, AuctionError> {
        let auction = self.repo.find_by_id(id).await?;
        if &auction.status != required {
            return Err(AuctionError::WrongState {
                current: auction.status,
                required: required.clone(),
            });
        }
        Ok(auction)
    }
}