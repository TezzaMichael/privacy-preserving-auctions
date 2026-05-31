"use client";
import { useState, useEffect } from "react";
import { toast } from "sonner";
import { api } from "@/lib/api";
import { useAuthStore } from "@/store/auth";
import { loadSecret } from "@/lib/crypto";
import { createLoserRangeProofWasm, createProofOfOpeningWasm } from "@/lib/wasm"; 
import type { SealedBid, BidSecret } from "@/types";
import Modal from "@/components/ui/Modal";

interface Props { 
  auctionId: string; 
  myBid: SealedBid; 
  winnerValue: number; 
  onClose: () => void; 
  onSuccess: () => void; 
}

export default function LoserProofModal({ auctionId, myBid, winnerValue, onClose, onSuccess }: Props) {
  const { token, secretKeyHex } = useAuthStore();
  const [secret, setSecret] = useState<BidSecret | null>(null);
  const [manualValue, setManualValue] = useState("");
  const [manualBlinding, setManualBlinding] = useState("");
  const [loading, setLoading] = useState(false);
  
  const hasCryptoKey = !!secretKeyHex;
  const isIdentical = secret !== null && secret.value === winnerValue;

  useEffect(() => { 
    setSecret(loadSecret(auctionId));
  }, [auctionId]);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!token || !secretKeyHex) return;

    const v = secret ? secret.value : parseInt(manualValue);
    const b = secret ? secret.blinding_hex : manualBlinding;

    setLoading(true);
    try {
      if (isIdentical) {
        const proofJson = await createProofOfOpeningWasm(v, b, myBid.commitment_hex);
        if (!proofJson) throw new Error("Unable to generate tie proof.");
        await api.proofs.revealWinner(token, auctionId, myBid.bid_id, v, proofJson);
      } else {
        const proofJson = await createLoserRangeProofWasm(v, b, winnerValue);
        if (!proofJson) throw new Error("Unable to generate ZKP.");
        await api.proofs.submitLoser(token, auctionId, myBid.bid_id, proofJson);
      }

      toast.success("Proof submitted and verified successfully.");
      onSuccess();
    } catch (err: any) {
      const msg = err.message?.toLowerCase();
      // BAN TRAP
      if (msg.includes("invalid") || msg.includes("proof")) {
          localStorage.setItem(`banned_${auctionId}`, "true");
          toast.error("Fraud detected! You have been disqualified.");
          onClose();
          window.location.reload(); 
      } else {
          toast.error(err.message || "Error submitting proof");
      }
    } finally {
      setLoading(false);
    }
  }

  return (
    <Modal title="Submit Proof of Loss" onClose={onClose}>
      {!hasCryptoKey ? (
        <div className="bg-yellow-500/10 border-l-4 border-yellow-500 p-4 mb-4 text-sm text-yellow-300 rounded">
          <p className="font-bold text-yellow-400 mb-1">Session Expired</p>
          <p>Please close this window, logout, and login again.</p>
        </div>
      ) : (
        <form onSubmit={handleSubmit} className="space-y-4">
          {secret ? (
            <div className="bg-slate-800 border border-slate-700 rounded-lg p-4 text-sm text-slate-300">
              <p className="mb-2">Your bid of <strong>{secret.value} ₿</strong> is ready for mathematical verification.</p>
              {isIdentical ? (
                <div className="bg-blue-500/10 border-l-2 border-blue-500 p-2 mt-2 text-xs text-blue-200">
                  <span className="font-bold">Tie Note:</span> Your bid is identical to the winner's, but their Bulletin Board timestamp is earlier. Submit the proof to accept the loss.
                </div>
              ) : (
                <p className="text-xs text-slate-500 mt-2">
                  The proof will mathematically demonstrate that your bid is lower than {winnerValue} ₿ without revealing it.
                </p>
              )}
            </div>
          ) : (
            <>
              <div className="bg-orange-500/10 border border-orange-500/20 rounded-lg p-3 text-sm text-orange-300">
                Secret not found. Enter data manually.
              </div>
              <div>
                <label className="block text-sm text-slate-400 mb-1">Bid Value</label>
                <input className="input" type="number" value={manualValue} onChange={e => setManualValue(e.target.value)} required={!secret} />
              </div>
              <div>
                <label className="block text-sm text-slate-400 mb-1">Blinding Hex</label>
                <input className="input mono" value={manualBlinding} onChange={e => setManualBlinding(e.target.value)} required={!secret} placeholder="64-char hex" />
              </div>
            </>
          )}
          <div className="flex gap-3 pt-2">
            <button type="submit" className="btn-primary flex-1 justify-center" disabled={loading}>
              {loading ? "Generating ZKP..." : "Submit Loser Proof"}
            </button>
            <button type="button" className="btn-secondary" onClick={onClose}>Cancel</button>
          </div>
        </form>
      )}
    </Modal>
  );
}