"use client";
import { useState } from "react";
import { RefreshCw, ChevronDown, ChevronUp } from "lucide-react";
import type { BBEntry } from "@/types";
import { shortHash, formatDate, entryKindColor } from "@/lib/utils";
import { cn } from "@/lib/utils";

interface Props { entries: BBEntry[]; onRefresh: () => void; }

export default function BulletinBoardPanel({ entries, onRefresh }: Props) {
  const [expanded, setExpanded] = useState<number | null>(null);

  return (
    <div className="card">
      <div className="flex items-center justify-between mb-4">
        <h2 className="font-semibold">Bulletin Board ({entries.length} entries)</h2>
        <button onClick={onRefresh} className="btn-ghost text-sm"><RefreshCw size={14} /> Refresh</button>
      </div>
      {entries.length === 0 ? (
        <p className="text-slate-500 text-sm text-center py-8">No entries</p>
      ) : (
        <div className="space-y-1">
          {entries.map(e => (
            <div key={e.sequence} className="rounded-lg bg-surface-border/30 overflow-hidden">
              <button
                className="w-full flex items-center gap-3 p-3 hover:bg-surface-border/50 transition-colors text-left"
                onClick={() => setExpanded(expanded === e.sequence ? null : e.sequence)}
              >
                <span className="text-slate-500 mono text-xs w-6 text-right">{e.sequence}</span>
                <span className={cn("text-xs font-medium w-36 shrink-0", entryKindColor(e.entry_kind))}>{e.entry_kind}</span>
                <span className="mono text-xs text-slate-500 flex-1 truncate">{shortHash(e.entry_hash_hex, 12)}</span>
                <span className="text-xs text-slate-600">{formatDate(e.recorded_at)}</span>
                {expanded === e.sequence ? <ChevronUp size={14} className="text-slate-500" /> : <ChevronDown size={14} className="text-slate-500" />}
              </button>
              {expanded === e.sequence && (
                <div className="border-t border-surface-border px-3 pb-3 pt-2 space-y-1">
                  <div className="grid grid-cols-2 gap-2 text-xs">
                    <div><span className="text-slate-500">prev_hash: </span><span className="mono text-slate-400">{shortHash(e.prev_hash_hex, 16)}</span></div>
                    <div><span className="text-slate-500">entry_hash: </span><span className="mono text-slate-400">{shortHash(e.entry_hash_hex, 16)}</span></div>
                    <div className="col-span-2"><span className="text-slate-500">sig: </span><span className="mono text-slate-400">{shortHash(e.server_signature_hex, 16)}</span></div>
                  </div>
                  <div className="mt-2">
                    <span className="text-slate-500 text-xs">payload: </span>
                    <pre className="mono text-xs text-slate-400 bg-surface/50 rounded p-2 mt-1 overflow-x-auto">
                      {JSON.stringify(JSON.parse(e.payload_json), null, 2)}
                    </pre>
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}