use wasm_bindgen::prelude::*;
use crate::transcript::{verify_auction_transcript, AuctionTranscript};

#[wasm_bindgen(start)]
pub fn wasm_init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn verify_transcript(transcript_json: &str) -> Result<JsValue, JsValue> {
    let t: AuctionTranscript = serde_json::from_str(transcript_json)
        .map_err(|e| JsValue::from_str(&format!("parse error: {e}")))?;
    let result = verify_auction_transcript(&t);
    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn verify_commitment(commitment_hex: &str, value: u64, blinding_hex: &str) -> bool {
    use auction_crypto::pedersen::{BlindingFactor, PedersenCommitment, PedersenGenerators};
    let gens = PedersenGenerators::standard();
    let Ok(c) = PedersenCommitment::from_hex(commitment_hex) else { return false; };
    let Some(r) = BlindingFactor::from_hex(blinding_hex) else { return false; };
    c.verify(value, &r, &gens)
}

#[wasm_bindgen]
pub fn verify_proof(proof_json: &str) -> bool {
    use auction_crypto::{pedersen::PedersenGenerators, schnorr::ProofOfOpening};
    let gens = PedersenGenerators::standard();
    let Ok(p) = serde_json::from_str::<ProofOfOpening>(proof_json) else { return false; };
    p.verify(&gens).is_ok()
}

#[wasm_bindgen]
pub fn verify_chain(entries_json: &str) -> Result<JsValue, JsValue> {
    use auction_core::bulletin_board::BulletinBoardEntry;
    use crate::bulletin_board::verify_chain_integrity;
    let entries: Vec<BulletinBoardEntry> = serde_json::from_str(entries_json)
        .map_err(|e| JsValue::from_str(&format!("parse error: {e}")))?;
    let result = match verify_chain_integrity(&entries) {
        Ok(_)  => serde_json::json!({"valid": true}),
        Err(e) => serde_json::json!({"valid": false, "error": e.to_string()}),
    };
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}