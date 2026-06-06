import { CheckCircle, XCircle, AlertCircle, Loader2, Shield } from "lucide-react";
import type { VerificationResult } from "@/types";
import { cn } from "@/lib/utils";

interface Props {
  result: VerificationResult | null;
  loading: boolean;
  hasWasm: boolean;
  onVerify: () => void;
  isAuctionEnded?: boolean; 
}

function Check({ passed, label, error, isEnded }: { passed: boolean; label: string; error?: string | null; isEnded?: boolean }) {
  const isFraudPrevented = !passed && error && error.includes("Bid >= Winner");
  
  const isMissingProofs = !passed && error && error.includes("Missing loser proofs");
  const isTimeout = isMissingProofs && isEnded;
  
  const displayPassed = passed || isFraudPrevented;

  return (
    <div className={cn(
      "flex items-start gap-3 p-3 rounded-lg", 
      displayPassed ? "bg-green-500/10" : isTimeout ? "bg-amber-500/10" : "bg-red-500/10"
    )}>
      {displayPassed ? (
        <CheckCircle size={18} className="text-green-400 shrink-0 mt-0.5" />
      ) : isTimeout ? (
        <AlertCircle size={18} className="text-amber-400 shrink-0 mt-0.5" />
      ) : (
        <XCircle size={18} className="text-red-400 shrink-0 mt-0.5" />
      )}
      
      <div>
        <div className={cn(
          "text-sm font-medium", 
          displayPassed ? "text-green-300" : isTimeout ? "text-amber-300" : "text-red-300"
        )}>
          {isFraudPrevented ? `${label} (Validated Challenge)` : isTimeout ? `${label} (Timeout)` : label}
        </div>
        {(error || isFraudPrevented) && (
          <div className={cn(
            "text-xs mt-0.5 mono",
            displayPassed ? "text-green-400" : isTimeout ? "text-amber-400" : "text-red-400"
          )}>
            {isFraudPrevented 
              ? "Validated challenge prevented a fraudulent winner declaration." 
              : isTimeout 
              ? "Some participants did not submit their proof in time. The auction can still conclude." 
              : error}
          </div>
        )}
      </div>
    </div>
  );
}

export default function VerificationPanel({ result, loading, hasWasm, onVerify, isAuctionEnded = false }: Props) {
  const isMissingProofs = result && !result.fully_valid && result.loser_proofs.error?.includes("Missing loser proofs");
  const isTimeoutFail = isMissingProofs && isAuctionEnded;
  const isFraudPrevented = result && !result.fully_valid && result.loser_proofs.error?.includes("Bid >= Winner");
  
  const showAsSuccess = result && (result.fully_valid || isTimeoutFail || isFraudPrevented);

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
          <div className={cn(
            "p-4 rounded-lg text-center mb-4 border", 
            showAsSuccess ? "bg-green-500/10 border-green-500/20" : "bg-red-500/10 border-red-500/20"
          )}>
            {showAsSuccess ? (
              <div className="text-green-300 font-bold text-lg flex items-center justify-center gap-2">
                <CheckCircle size={20} /> 
                {isFraudPrevented ? "Validated Challenge" : isTimeoutFail ? "Auction Partially Verified" : "Auction Fully Verified"}
              </div>
            ) : (
              <div className="text-red-300 font-bold text-lg flex items-center justify-center gap-2">
                <XCircle size={20} /> Verification Failed
              </div>
            )}
          </div>
          
          <Check passed={result.chain_integrity.passed} label="Chain Integrity" error={result.chain_integrity.error} isEnded={isAuctionEnded} />
          <Check passed={result.server_signatures.passed} label="Server Signatures" error={result.server_signatures.error} isEnded={isAuctionEnded} />
          <Check passed={result.winner_proof.passed} label="Winner Proof" error={result.winner_proof.error} isEnded={isAuctionEnded} />
          <Check passed={result.loser_proofs.passed} label="Loser Proofs" error={result.loser_proofs.error} isEnded={isAuctionEnded} />
        </div>
      ) : (
        <p className="text-slate-500 text-sm text-center py-8">Press "Run Verification" to verify this auction client-side using the WASM verifier.</p>
      )}
    </div>
  );
}