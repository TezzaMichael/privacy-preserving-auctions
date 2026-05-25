import { Trophy, CheckCircle, XCircle } from "lucide-react";
import type { WinnerReveal, LoserProof, SealedBid } from "@/types";
import { shortHash } from "@/lib/utils";

interface Props { winner: WinnerReveal | null; losers: LoserProof[]; bids: SealedBid[]; }

export default function ProofsPanel({ winner, losers, bids }: Props) {
  return (
    <div className="card">
      <h2 className="font-semibold mb-4 flex items-center gap-2">
        <Trophy size={16} className="text-yellow-400" />
        Reveals & Proofs
      </h2>
      {!winner && losers.length === 0 ? (
        <p className="text-slate-500 text-sm text-center py-8">No reveals yet</p>
      ) : (
        <div className="space-y-3">
          {winner && (
            <div className="p-3 rounded-lg bg-yellow-500/10 border border-yellow-500/20">
              <div className="flex items-center gap-2 mb-1">
                <Trophy size={14} className="text-yellow-400" />
                <span className="text-yellow-300 text-sm font-medium">Winner</span>
              </div>
              <div className="mono text-slate-300 text-sm">{shortHash(winner.winner_id)}</div>
              <div className="text-xl font-bold mt-1">{winner.revealed_value.toLocaleString()}</div>
              <div className="mono text-xs text-slate-500 mt-1">BB #{winner.bb_sequence}</div>
            </div>
          )}
          {losers.map(l => (
            <div key={l.proof_id} className="p-3 rounded-lg bg-surface-border/50">
              <div className="flex items-center justify-between">
                <div>
                  <div className="mono text-slate-400 text-xs">{shortHash(l.bidder_id)}</div>
                  <div className="font-semibold">{l.revealed_value.toLocaleString()}</div>
                </div>
                <div className="flex items-center gap-1">
                  {l.verified
                    ? <><CheckCircle size={14} className="text-green-400" /><span className="text-green-400 text-xs">Verified</span></>
                    : <><XCircle size={14} className="text-red-400" /><span className="text-red-400 text-xs">Pending</span></>}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}