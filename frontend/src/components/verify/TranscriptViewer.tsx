"use client";
import { useState } from "react";
import { ChevronDown, ChevronUp } from "lucide-react";
import type { BBEntry, SealedBid, WinnerReveal, LoserProof } from "@/types";
import { shortHash, entryKindColor, formatDate } from "@/lib/utils";
import { cn } from "@/lib/utils";

interface Props {
  entries: BBEntry[];
  bids: SealedBid[];
  winner: WinnerReveal | null;
  losers: LoserProof[];
}

export default function TranscriptViewer({ entries, bids, winner, losers }: Props) {
  const [tab, setTab] = useState<"chain" | "bids" | "proofs">("chain");

  return (
    <div className="card">
      <h2 className="font-semibold mb-4">Auction Transcript</h2>
      <div className="flex gap-1 mb-4 bg-surface-border/50 p-1 rounded-lg w-fit">
        {(["chain", "bids", "proofs"] as const).map(t => (
          <button key={t} onClick={() => setTab(t)} className={cn("px-3 py-1.5 rounded text-sm capitalize transition-colors", tab === t ? "bg-brand text-white" : "text-slate-400 hover:text-slate-200")}>
            {t === "chain" ? `Chain (${entries.length})` : t === "bids" ? `Bids (${bids.length})` : `Proofs (${1 + losers.length})`}
          </button>
        ))}
      </div>
      {tab === "chain" && (
        <div className="space-y-1 max-h-96 overflow-y-auto">
          {entries.map(e => (
            <div key={e.sequence} className="flex items-center gap-3 p-2 rounded bg-surface-border/30 text-xs">
              <span className="text-slate-500 mono w-4 text-right shrink-0">{e.sequence}</span>
              <span className={cn("w-32 shrink-0 font-medium", entryKindColor(e.entry_kind))}>{e.entry_kind}</span>
              <span className="mono text-slate-500 flex-1 truncate">{shortHash(e.entry_hash_hex, 16)}</span>
              <span className="text-slate-600">{formatDate(e.recorded_at)}</span>
            </div>
          ))}
        </div>
      )}
      {tab === "bids" && (
        <div className="space-y-1 max-h-96 overflow-y-auto">
          {bids.map(b => (
            <div key={b.bid_id} className="p-2 rounded bg-surface-border/30 text-xs">
              <div className="flex items-center justify-between">
                <span className="mono text-slate-400">{shortHash(b.bidder_id)}</span>
                {b.bb_sequence !== null && <span className="text-slate-500">BB #{b.bb_sequence}</span>}
              </div>
              <div className="mono text-slate-600 mt-0.5">C = {shortHash(b.commitment_hex, 20)}</div>
            </div>
          ))}
        </div>
      )}
      {tab === "proofs" && (
        <div className="space-y-2 max-h-96 overflow-y-auto">
          {winner && (
            <div className="p-3 rounded bg-yellow-500/10 text-xs">
              <div className="text-yellow-300 font-medium mb-1">Winner — value: {winner.revealed_value}</div>
              <div className="mono text-slate-500">{shortHash(winner.winner_id)}</div>
            </div>
          )}
          {losers.map(l => (
            <div key={l.proof_id} className="p-3 rounded bg-surface-border/30 text-xs">
              <div className="flex items-center justify-between">
                <div>
                  <div className="mono text-slate-400">{shortHash(l.bidder_id)}</div>
                  <div className="text-slate-300 font-medium">value: {l.revealed_value}</div>
                </div>
                <span className={cn("badge", l.verified ? "bg-green-500/20 text-green-300" : "bg-orange-500/20 text-orange-300")}>
                  {l.verified ? "Verified" : "Pending"}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}