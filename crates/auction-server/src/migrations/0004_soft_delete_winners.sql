-- Create the new table without the UNIQUE constraint on auction_id, and add is_valid
CREATE TABLE winner_reveals_new (
    id              TEXT PRIMARY KEY,
    auction_id      TEXT NOT NULL REFERENCES auctions(id),
    winner_id       TEXT NOT NULL REFERENCES users(id),
    bid_id          TEXT NOT NULL REFERENCES sealed_bids(id),
    revealed_value  INTEGER NOT NULL,
    proof_json      TEXT NOT NULL,
    bb_sequence     INTEGER,
    submitted_at    TEXT NOT NULL,
    is_valid        INTEGER NOT NULL DEFAULT 1
);

-- Copy the existing data over
INSERT INTO winner_reveals_new 
    (id, auction_id, winner_id, bid_id, revealed_value, proof_json, bb_sequence, submitted_at)
SELECT 
    id, auction_id, winner_id, bid_id, revealed_value, proof_json, bb_sequence, submitted_at 
FROM winner_reveals;

-- Drop the old table and rename the new one
DROP TABLE winner_reveals;
ALTER TABLE winner_reveals_new RENAME TO winner_reveals;

-- Enforce that there can only be ONE active winner per auction using a partial index
CREATE UNIQUE INDEX idx_unique_valid_winner ON winner_reveals(auction_id) WHERE is_valid = 1;
