import { Shield } from "lucide-react";
import type { SealedBid } from "@/types";
import { shortHash, formatDate, cn } from "@/lib/utils";

interface Props { bids: SealedBid[]; currentUserId?: string; }

export default function BidsPanel({ bids, currentUserId }: Props) {
  return (
    <div className="card">
      <h2 className="font-semibold mb-4 flex items-center gap-2">
        <Shield size={16} className="text-brand" />
        Sealed Bids ({bids.length})
      </h2>
      {bids.length === 0 ? (
        <p className="text-slate-500 text-sm text-center py-8">No bids yet</p>
      ) : (
        <div className="space-y-2">
          {bids.map(b => (
            <div key={b.bid_id} className={cn("flex items-center justify-between p-3 rounded-lg bg-surface-border/50", b.bidder_id === currentUserId && "border border-brand/30")}>
              <div>
                <div className="flex items-center gap-2">
                  <span className="mono text-slate-400">{shortHash(b.bidder_id)}</span>
                  {b.bidder_id === currentUserId && <span className="badge bg-brand/20 text-brand text-xs">You</span>}
                </div>
                <div className="mono text-xs text-slate-600 mt-0.5">C: {shortHash(b.commitment_hex)}</div>
              </div>
              <div className="text-right">
                {b.bb_sequence !== null && <div className="text-xs text-slate-500">BB #{b.bb_sequence}</div>}
                <div className="text-xs text-slate-600">{formatDate(b.submitted_at)}</div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}