export type AuctionStatus = "Pending" | "BiddingOpen" | "ClaimPhase" | "ProofPhase" | "Closed";

export interface User {
  user_id: string;
  username: string;
  public_key_hex: string;
  created_at?: string;
}

export interface Auction {
  id: string;
  creator_id: string;
  title: string;
  description: string;
  status: AuctionStatus;
  min_bid: number;
  max_bid: number;
  bid_step: number;
  end_time: string;
  bb_create_sequence: number | null;
  created_at: string;
  updated_at: string;
}

export interface SealedBid {
  bid_id: string;
  bidder_id: string;
  commitment_hex: string;
  bidder_signature_hex: string;
  bb_sequence: number | null;
  submitted_at: string;
}

export interface BBEntry {
  sequence: number;
  auction_id: string;
  entry_kind: string;
  payload_json: string;
  prev_hash_hex: string;
  entry_hash_hex: string;
  server_signature_hex: string;
  recorded_at: string;
}

export interface WinnerReveal {
  reveal_id: string;
  auction_id: string;
  winner_id: string;
  bid_id: string;
  revealed_value: number;
  proof_json: string;
  bb_sequence: number | null;
  submitted_at: string;
}

export interface LoserProof {
  proof_id: string;
  bidder_id: string;
  bid_id: string;
  revealed_value: number;
  proof_json: string;
  verified: boolean;
  bb_sequence: number | null;
  submitted_at: string;
}

export interface ServerPublicKey {
  public_key_hex: string;
  pedersen_g_hex: string;
  pedersen_h_hex: string;
}

export interface VerificationResult {
  auction_id: string;
  chain_integrity: CheckResult;
  server_signatures: CheckResult;
  winner_proof: CheckResult;
  loser_proofs: CheckResult;
  fully_valid: boolean;
}

export interface CheckResult {
  passed: boolean;
  error: string | null;
}

export interface BidSecret {
  auction_id: string;
  value: number;
  blinding_hex: string;
  commitment_hex: string;
}