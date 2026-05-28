import { CheckCircle, ShieldCheck, Trophy, XCircle } from "lucide-react";
import type { WinnerReveal, LoserProof, SealedBid } from "@/types";
import { formatAddress } from "@/lib/utils";

interface Props {
  winner: WinnerReveal | null;
  losers: LoserProof[];
  bids: SealedBid[];
}

export default function ProofsPanel({ winner, losers }: Props) {
  return (
    <div className="card h-full flex flex-col">
      <h2 className="text-lg font-bold mb-4 flex items-center gap-2">
        <ShieldCheck className="text-brand-light" size={20} />
        Reveals & Proofs
      </h2>

      <div className="space-y-4 flex-1">
        
        {/* VINCITORE: Mostra il valore in chiaro */}
        {winner ? (
          <div className="p-4 rounded-xl border border-amber-500/30 bg-amber-500/10 shadow-[0_0_15px_rgba(245,158,11,0.1)]">
            <div className="flex items-center gap-2 text-amber-400 font-bold mb-3 uppercase tracking-wider text-sm">
              <Trophy size={18} /> Verified Winner
            </div>
            <div className="flex justify-between items-end">
              <div>
                <div className="text-sm text-slate-400">Bidder:</div>
                <div className="font-mono text-slate-200">{formatAddress(winner.winner_id)}</div>
                <div className="font-mono text-xs text-slate-500 mt-1">BB #{winner.bb_sequence}</div>
              </div>
              <div className="text-right">
                <div className="text-3xl font-bold font-mono text-white">
                  {winner.revealed_value.toLocaleString()} ₿
                </div>
              </div>
            </div>
          </div>
        ) : (
          <div className="text-sm text-slate-500 p-6 border border-dashed border-slate-700/50 rounded-xl text-center bg-slate-900/30">
            No winner revealed yet.
          </div>
        )}

        {/* PERDENTI: Nasconde il valore per privacy, mostra solo la validazione ZK */}
        {losers && losers.length > 0 && (
          <div className="mt-6">
            <h3 className="text-xs font-semibold text-slate-500 mb-3 uppercase tracking-wider">
              Validated Cryptographic Proofs
            </h3>
            <div className="space-y-2">
              {losers.map(l => (
                <div key={l.proof_id} className="p-3 rounded-lg bg-slate-800/50 border border-slate-700/50">
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-mono text-slate-400 text-xs">
                        Bidder: {formatAddress(l.bidder_id)}
                      </div>
                      <div className="font-semibold text-emerald-500 italic text-sm mt-1">
                        Hidden (ZK Proof)
                      </div>
                    </div>
                    <div className="flex items-center gap-1">
                      {l.verified ? (
                        <>
                          <CheckCircle size={16} className="text-emerald-400" />
                          <span className="text-emerald-400 text-xs font-medium">Verified</span>
                        </>
                      ) : (
                        <>
                          <XCircle size={16} className="text-amber-400" />
                          <span className="text-amber-400 text-xs font-medium">Pending</span>
                        </>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

      </div>
    </div>
  );
}