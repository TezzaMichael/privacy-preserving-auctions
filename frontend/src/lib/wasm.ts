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

// All'interno di wasm.ts, aggiorna SOLO la funzione verifyTranscriptWasm:
export async function verifyTranscriptWasm(transcriptJson: string): Promise<VerificationResult | null> {
  const mod = await loadWasm();
  if (!mod) {
    console.error("Modulo WASM non caricato!");
    return null;
  }
  try {
    console.log("Inviando questi dati a Rust per la verifica:", JSON.parse(transcriptJson));
    const result = mod.verify_transcript(transcriptJson);
    return result as VerificationResult;
  } catch (err) {
    // Questo è il log fondamentale che ci serve!
    console.error("ERRORE DI VALIDAZIONE IN RUST (verify_transcript):", err);
    return null;
  }
}

export async function verifyChainWasm(entries: BBEntry[]): Promise<{ valid: boolean; error?: string } | null> {
  const mod = await loadWasm();
  if (!mod) return null;
  try {
    return mod.verify_chain(JSON.stringify(entries));
  } catch {
    return null;
  }
}

export async function verifyCommitmentWasm(
  commitmentHex: string, value: number, blindingHex: string
): Promise<boolean> {
  const mod = await loadWasm();
  if (!mod) return false;
  try {
    return mod.verify_commitment(commitmentHex, BigInt(value), blindingHex);
  } catch {
    return false;
  }
}

export async function verifyProofWasm(proofJson: string): Promise<boolean> {
  const mod = await loadWasm();
  if (!mod) return false;
  try {
    return mod.verify_proof(proofJson);
  } catch {
    return false;
  }
}

export const wasmAvailable = async (): Promise<boolean> => !!(await loadWasm());

export async function createCommitmentWasm(value: number, blindingHex: string): Promise<string | null> {
  const mod = await loadWasm();
  if (!mod) {
    console.error("Il modulo WASM non si è caricato per niente.");
    return null;
  }
  
  // LOG DI DEBUG UTILI:
  console.log("WASM Mod caricato:", mod);
  console.log("Esiste create_commitment?", typeof mod.create_commitment);

  try {
    return mod.create_commitment(BigInt(value), blindingHex);
  } catch (err) {
    console.error("ERRORE INTERNO WASM create_commitment:", err);
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