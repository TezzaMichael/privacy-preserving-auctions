import type { BidSecret } from "@/types";
import { ed25519 } from '@noble/curves/ed25519.js';

function randomBytes(n: number): Uint8Array {
  return crypto.getRandomValues(new Uint8Array(n));
}

function toHex(bytes: Uint8Array): string {
  return Array.from(bytes).map(b => b.toString(16).padStart(2, "0")).join("");
}

function fromHex(hex: string): Uint8Array {
  const arr = new Uint8Array(hex.length / 2);
  for (let i = 0; i < arr.length; i++) arr[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  return arr;
}

async function sha256(data: Uint8Array): Promise<Uint8Array> {
  const buf = await crypto.subtle.digest("SHA-256", data as BufferSource);
  return new Uint8Array(buf);
}

function encodeLength(n: number, bytes: 4 | 8): Uint8Array {
  const buf = new ArrayBuffer(bytes);
  const view = new DataView(buf);
  if (bytes === 4) view.setUint32(0, n, true);
  else { view.setUint32(0, n & 0xffffffff, true); view.setUint32(4, 0, true); }
  return new Uint8Array(buf);
}

// FORMATO CRITTOGRAFICO RIGOROSO RICHIESTO DAL BACKEND RUST
async function commitmentMessage(auctionIdBytes: Uint8Array, commitmentHex: string): Promise<Uint8Array> {
  const domain = new TextEncoder().encode("auction-bid-commitment-v1:");
  const chBytes = new TextEncoder().encode(commitmentHex);
  const parts = [
    domain,
    encodeLength(auctionIdBytes.length, 8),
    auctionIdBytes,
    encodeLength(chBytes.length, 8),
    chBytes,
  ];
  const total = parts.reduce((s, p) => s + p.length, 0);
  const buf = new Uint8Array(total);
  let off = 0;
  for (const p of parts) { buf.set(p, off); off += p.length; }
  
  // Hash SHA-256 essenziale prima della firma
  return sha256(buf);
}

export function generateBidSecret(auctionId: string, value: number): BidSecret {
  return {
    auction_id: auctionId,
    value,
    blinding_hex: toHex(randomBytes(32)),
    commitment_hex: "", // Aggiunto per soddisfare TypeScript
  };
}

export async function signCommitment(
  secretKeyHex: string,
  auctionId: string,
  commitmentHex: string
): Promise<string> {
  // Il server vuole l'ID dell'asta trasformato in byte rimuovendo i trattini
  const auctionIdBytes = fromHex(auctionId.replace(/-/g, ""));
  
  const msg = await commitmentMessage(auctionIdBytes, commitmentHex);
  const privateKeyBytes = fromHex(secretKeyHex);
  const signature = ed25519.sign(msg, privateKeyBytes);
  
  return toHex(signature);
}

export async function deriveKeypairFromPassword(password: string, username: string): Promise<{ publicKeyHex: string, secretKeyHex: string }> {
  const enc = new TextEncoder();
  
  const keyMaterial = await crypto.subtle.importKey(
    "raw",
    enc.encode(password),
    { name: "PBKDF2" },
    false,
    ["deriveBits"]
  );

  const salt = enc.encode(`auction_salt_${username.toLowerCase()}`);
  
  const derivedBits = await crypto.subtle.deriveBits(
    {
      name: "PBKDF2",
      salt: salt,
      iterations: 100000, 
      hash: "SHA-256"
    },
    keyMaterial,
    256 
  );

  const privKey = new Uint8Array(derivedBits);
  const pubKey = ed25519.getPublicKey(privKey);

  return {
    secretKeyHex: toHex(privKey),
    publicKeyHex: toHex(pubKey),
  };
}

export function storeSecret(secret: BidSecret): void {
  const key = `bid_secret_${secret.auction_id}`;
  sessionStorage.setItem(key, JSON.stringify(secret));
}

export function loadSecret(auctionId: string): BidSecret | null {
  const raw = sessionStorage.getItem(`bid_secret_${auctionId}`);
  return raw ? JSON.parse(raw) : null;
}

export function listSecretAuctionIds(): string[] {
  return Object.keys(sessionStorage)
    .filter(k => k.startsWith("bid_secret_"))
    .map(k => k.replace("bid_secret_", ""));
}