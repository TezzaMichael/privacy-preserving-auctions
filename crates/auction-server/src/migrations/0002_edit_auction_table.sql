-- update auction table to include new fields for bidding, auction end time and remove reserve_price
ALTER TABLE auctions DROP COLUMN reserve_price;
ALTER TABLE auctions ADD COLUMN min_bid INTEGER NOT NULL DEFAULT 0;
ALTER TABLE auctions ADD COLUMN max_bid INTEGER;
ALTER TABLE auctions ADD COLUMN bid_step INTEGER NOT NULL DEFAULT 1;
ALTER TABLE auctions ADD COLUMN end_time TEXT NOT NULL DEFAULT '2026-12-31T23:59:59Z';