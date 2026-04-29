import { CheckCircle, XCircle, AlertCircle, Loader2, Shield } from "lucide-react";
import type { VerificationResult } from "@/types";
import { cn } from "@/lib/utils";

interface Props {
  result: VerificationResult | null;
  loading: boolean;
  hasWasm: boolean;
  onVerify: () => void;
}

function Check({ passed, label, error }: { passed: boolean; label: string; error?: string | null }) {
  return (
    <div className={cn("flex items-start gap-3 p-3 rounded-lg", passed ? "bg-green-500/10" : "bg-red-500/10")}>
      {passed
        ? <CheckCircle size={18} className="text-green-400 shrink-0 mt-0.5" />
        : <XCircle size={18} className="text-red-400 shrink-0 mt-0.5" />}
      <div>
        <div className={cn("text-sm font-medium", passed ? "text-green-300" : "text-red-300")}>{label}</div>
        {error && <div className="text-xs text-red-400 mt-0.5 mono">{error}</div>}
      </div>
    </div>
  );
}

export default function VerificationPanel({ result, loading, hasWasm, onVerify }: Props) {
  return (
    <div className="card">
      <div className="flex items-center justify-between mb-4">
        <h2 className="font-semibold flex items-center gap-2"><Shield size={16} className="text-brand" /> Zero-Trust Verification</h2>
        <button className="btn-primary" onClick={onVerify} disabled={loading || !hasWasm}>
          {loading ? <><Loader2 size={14} className="animate-spin" /> Verifying…</> : "Run Verification"}
        </button>
      </div>
      {!hasWasm && (
        <div className="bg-orange-500/10 border border-orange-500/20 rounded-lg p-3 text-sm text-orange-300 mb-4 flex items-center gap-2">
          <AlertCircle size={14} /> WASM verifier not loaded. Run <code className="mono bg-surface/50 px-1 rounded">wasm-pack build</code> to enable client-side verification.
        </div>
      )}
      {result ? (
        <div className="space-y-2">
          <div className={cn("p-4 rounded-lg text-center mb-4", result.fully_valid ? "bg-green-500/10 border border-green-500/20" : "bg-red-500/10 border border-red-500/20")}>
            {result.fully_valid
              ? <div className="text-green-300 font-bold text-lg flex items-center justify-center gap-2"><CheckCircle size={20} /> Auction Fully Verified</div>
              : <div className="text-red-300 font-bold text-lg flex items-center justify-center gap-2"><XCircle size={20} /> Verification Failed</div>}
          </div>
          <Check passed={result.chain_integrity.passed} label="Chain Integrity" error={result.chain_integrity.error} />
          <Check passed={result.server_signatures.passed} label="Server Signatures" error={result.server_signatures.error} />
          <Check passed={result.winner_proof.passed} label="Winner Proof" error={result.winner_proof.error} />
          <Check passed={result.loser_proofs.passed} label="Loser Proofs" error={result.loser_proofs.error} />
        </div>
      ) : (
        <p className="text-slate-500 text-sm text-center py-8">Press "Run Verification" to verify this auction client-side using the WASM verifier.</p>
      )}
    </div>
  );
}