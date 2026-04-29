import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import type { AuctionStatus } from "@/types";

export function cn(...inputs: ClassValue[]) { return twMerge(clsx(inputs)); }

export function shortHash(hex: string, n = 8): string {
  return hex.length > n * 2 ? `${hex.slice(0, n)}…${hex.slice(-n)}` : hex;
}

export function formatDate(iso: string): string {
  return new Date(iso).toLocaleString();
}

export function statusColor(s: AuctionStatus): string {
  return {
    Pending:     "bg-slate-500/20 text-slate-300",
    BiddingOpen: "bg-green-500/20 text-green-300",
    ClaimPhase:  "bg-yellow-500/20 text-yellow-300",
    ProofPhase:  "bg-orange-500/20 text-orange-300",
    Closed:      "bg-red-500/20 text-red-300",
  }[s] ?? "bg-slate-500/20 text-slate-300";
}

export function statusLabel(s: AuctionStatus): string {
  return { Pending: "Pending", BiddingOpen: "Bidding Open", ClaimPhase: "Claim Phase", ProofPhase: "Proof Phase", Closed: "Closed" }[s] ?? s;
}

export function entryKindColor(kind: string): string {
  return {
    AuctionCreate:   "text-blue-400",
    AuctionOpen:     "text-green-400",
    SealedBid:       "text-purple-400",
    AuctionClose:    "text-yellow-400",
    WinnerReveal:    "text-orange-400",
    LoserProof:      "text-pink-400",
    ProofCertificate:"text-cyan-400",
    AuctionFinalize: "text-red-400",
  }[kind] ?? "text-slate-400";
}