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
  if (!mod) return null;
  try {
    const result = mod.verify_transcript(transcriptJson);
    return result as VerificationResult;
  } catch {
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