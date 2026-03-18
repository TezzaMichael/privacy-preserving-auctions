use sha2::{Sha256, Digest};
use rand::{Rng, thread_rng};

// Generation of a random nonce (128-bit)
pub fn generate_nonce() -> String {
    let mut rng = thread_rng();
    let random_bytes: [u8; 16] = rng.r#gen(); // 128-bit nonce
    hex::encode(random_bytes)
}

// Commitment = hash(bid || nonce)
pub fn commit(bid: u64, nonce: &str) -> String {
    let mut hasher = Sha256::new();

    let input = format!("{}{}", bid, nonce);
    hasher.update(input);

    let result = hasher.finalize();
    hex::encode(result)
}

// Verification: hash(bid || nonce) == commitment
pub fn verify(commitment: &str, bid: u64, nonce: &str) -> bool {
    let computed = commit(bid, nonce);
    computed == commitment
}