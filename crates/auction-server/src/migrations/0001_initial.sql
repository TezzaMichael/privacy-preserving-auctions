CREATE TABLE IF NOT EXISTS users (
    id              TEXT PRIMARY KEY,
    username        TEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    public_key_hex  TEXT NOT NULL,
    created_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS auctions (
    id                    TEXT PRIMARY KEY,
    creator_id            TEXT NOT NULL REFERENCES users(id),
    title                 TEXT NOT NULL,
    description           TEXT NOT NULL,
    status                TEXT NOT NULL DEFAULT 'Pending',
    reserve_price         INTEGER,
    server_signature_hex  TEXT,
    bb_create_sequence    INTEGER,
    created_at            TEXT NOT NULL,
    updated_at            TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sealed_bids (
    id                    TEXT PRIMARY KEY,
    auction_id            TEXT NOT NULL REFERENCES auctions(id),
    bidder_id             TEXT NOT NULL REFERENCES users(id),
    commitment_hex        TEXT NOT NULL,
    bidder_signature_hex  TEXT NOT NULL,
    bb_sequence           INTEGER,
    submitted_at          TEXT NOT NULL,
    UNIQUE(auction_id, bidder_id)
);

CREATE TABLE IF NOT EXISTS bulletin_board (
    sequence              INTEGER NOT NULL,
    auction_id            TEXT NOT NULL REFERENCES auctions(id),
    entry_kind            TEXT NOT NULL,
    payload_json          TEXT NOT NULL,
    prev_hash_hex         TEXT NOT NULL,
    entry_hash_hex        TEXT NOT NULL,
    server_signature_hex  TEXT NOT NULL,
    recorded_at           TEXT NOT NULL,
    PRIMARY KEY (auction_id, sequence)
);

CREATE TABLE IF NOT EXISTS winner_reveals (
    id              TEXT PRIMARY KEY,
    auction_id      TEXT NOT NULL REFERENCES auctions(id) UNIQUE,
    winner_id       TEXT NOT NULL REFERENCES users(id),
    bid_id          TEXT NOT NULL REFERENCES sealed_bids(id),
    revealed_value  INTEGER NOT NULL,
    proof_json      TEXT NOT NULL,
    bb_sequence     INTEGER,
    submitted_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS loser_proofs (
    id              TEXT PRIMARY KEY,
    auction_id      TEXT NOT NULL REFERENCES auctions(id),
    bidder_id       TEXT NOT NULL REFERENCES users(id),
    bid_id          TEXT NOT NULL REFERENCES sealed_bids(id),
    revealed_value  INTEGER NOT NULL,
    proof_json      TEXT NOT NULL,
    verified        INTEGER NOT NULL DEFAULT 0,
    bb_sequence     INTEGER,
    submitted_at    TEXT NOT NULL,
    UNIQUE(auction_id, bidder_id)
);

CREATE INDEX IF NOT EXISTS idx_auctions_creator    ON auctions(creator_id);
CREATE INDEX IF NOT EXISTS idx_bids_auction        ON sealed_bids(auction_id);
CREATE INDEX IF NOT EXISTS idx_bb_auction          ON bulletin_board(auction_id, sequence);
CREATE INDEX IF NOT EXISTS idx_loser_proofs_auction ON loser_proofs(auction_id);