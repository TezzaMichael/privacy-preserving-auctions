import type { VerificationResult, BBEntry } from "@/types";

let wasmModule: any = null;

export async function loadWasm(): Promise<any> {
  if (wasmModule) return wasmModule;
  try {
    const mod = await import("@/wasm/auction_verifier.js");
    await mod.default();
    wasmModule = mod;
    return mod;
  } catch {
    return null;
  }
}

export async function verifyTranscriptWasm(transcriptJson: string): Promise<VerificationResult | null> {
  const mod = await loadWasm();
  if (!mod) {
    console.error("WASM module not loaded!");
    return null;
  }
  try {
    console.log("Sending these data to Rust for verification:", JSON.parse(transcriptJson));
    const result = mod.verify_transcript(transcriptJson);
    return result as VerificationResult;
  } catch (err) {
    console.error("Error occurred while verifying transcript in Rust:", err);
    return null;
  }
}



export const wasmAvailable = async (): Promise<boolean> => !!(await loadWasm());

export async function createCommitmentWasm(value: number, blindingHex: string): Promise<string | null> {
  const mod = await loadWasm();
  if (!mod) {
    console.error("WASM module not loaded!");
    return null;
  }
  
  console.log("WASM Module loaded:", mod);
  console.log("Does create_commitment exist in the module?", typeof mod.create_commitment);

  try {
    return mod.create_commitment(BigInt(value), blindingHex);
  } catch (err) {
    console.error("Error occurred in WASM create_commitment:", err);
    return null;
  }
}

export async function createProofOfOpeningWasm(
  value: number,
  blindingHex: string,
  commitmentHex: string
): Promise<string | null> {
  const mod = await loadWasm();
  if (!mod) return null;
  try {
    return mod.create_proof_of_opening(BigInt(value), blindingHex, commitmentHex);
  } catch {
    return null;
  }
}

export async function createLoserRangeProofWasm(
  bidValue: number,
  blindingHex: string,
  winnerValue: number
): Promise<string | null> {
  const mod = await loadWasm();
  if (!mod) {
    console.error("WASM module not loaded!");
    return null;
  }
  
  try {
    // Call the new Rust WASM function
    return mod.create_loser_range_proof(BigInt(bidValue), blindingHex, BigInt(winnerValue));
  } catch (err) {
    console.error("Error occurred in WASM create_loser_range_proof:", err);
    return null;
  }
}
