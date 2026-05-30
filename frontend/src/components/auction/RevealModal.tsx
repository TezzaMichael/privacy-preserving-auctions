"use client";
import { useState, useEffect, useRef } from "react";
import { toast } from "sonner";
import { api } from "@/lib/api";
import { useAuthStore } from "@/store/auth";
import { loadSecret } from "@/lib/crypto";
import { createProofOfOpeningWasm } from "@/lib/wasm";
import type { SealedBid, BidSecret } from "@/types";
import Modal from "@/components/ui/Modal";

interface Props { 
  auctionId: string; 
  myBid: SealedBid; 
  currentWinnerValue?: number; 
  onClose: () => void; 
  onSuccess: () => void; 
}

export default function RevealModal({ auctionId, myBid, currentWinnerValue, onClose, onSuccess }: Props) {
  const { token, secretKeyHex } = useAuthStore();
  const [secret, setSecret] = useState<BidSecret | null>(null);
  const [manualValue, setManualValue] = useState("");
  const [manualBlinding, setManualBlinding] = useState("");
  const [loading, setLoading] = useState(false);
  
  const [autoSubmitting, setAutoSubmitting] = useState(false);
  const autoSubmitAttempted = useRef(false);

  const hasCryptoKey = !!secretKeyHex;
  const isChallenge = currentWinnerValue !== undefined && secret !== null && secret.value > currentWinnerValue;

  useEffect(() => { 
    const loadedSecret = loadSecret(auctionId);
    setSecret(loadedSecret);

    if (loadedSecret && hasCryptoKey && !autoSubmitAttempted.current && currentWinnerValue === undefined) {
      autoSubmitAttempted.current = true;
      setAutoSubmitting(true);
      
      setTimeout(() => {
        performReveal(loadedSecret.value, loadedSecret.blinding_hex);
      }, 1500);
    }
  }, [auctionId, hasCryptoKey, currentWinnerValue]);

  async function performReveal(value: number, blinding: string) {
    if (!token || !secretKeyHex) {
      toast.error("Invalid cryptographic session. Please logout and login again.");
      return;
    }

    setLoading(true);
    try {
      const realProof = await createProofOfOpeningWasm(value, blinding, myBid.commitment_hex);
      if (!realProof) {
        throw new Error("Unable to generate cryptographic proof via WASM.");
      }

      await api.proofs.revealWinner(token, auctionId, myBid.bid_id, value, realProof);
      toast.success(isChallenge ? "Challenge successful! You are the new winner." : "Claim submitted successfully!");
      onSuccess();
    } catch (err: any) {
      if (err.message.includes("fuori sincrono") || err.message.includes("out of sync")) {
        toast.error("Out of sync with the polling radar. Please try again.");
      } else {
        toast.error(err.message);
      }
    } finally {
      setLoading(false);
      setAutoSubmitting(false);
    }
  }

  async function handleManualReveal(e: React.FormEvent) {
    e.preventDefault();
    const v = secret ? secret.value : parseInt(manualValue);
    const b = secret ? secret.blinding_hex : manualBlinding;
    await performReveal(v, b);
  }

  return (
    <Modal title={isChallenge ? "Challenge Winner" : "Reveal Bid Claim"} onClose={onClose}>
      {!hasCryptoKey ? (
        <div className="bg-yellow-500/10 border-l-4 border-yellow-500 p-4 mb-4 text-sm text-yellow-300 rounded">
          <p className="font-bold text-yellow-400 mb-1">Session Expired</p>
          <p>Please close this window, logout, and login again.</p>
        </div>
      ) : autoSubmitting ? (
        <div className="py-8 text-center flex flex-col items-center space-y-4">
          <div className="w-12 h-12 border-4 border-blue-500/30 border-t-blue-500 rounded-full animate-spin"></div>
          <div>
            <h3 className="text-lg font-medium text-slate-200">Processing Automatically...</h3>
            <p className="text-sm text-slate-400 mt-2">Generating cryptographic proof.</p>
          </div>
        </div>
      ) : (
        <form onSubmit={handleManualReveal} className="space-y-4">
          {isChallenge ? (
            <div className="bg-purple-500/10 border border-purple-500/30 shadow-inner shadow-purple-500/10 rounded-xl p-4 text-sm text-purple-200">
              <h3 className="font-bold text-purple-400 text-base mb-2 flex items-center gap-2">
                🚨 Challenge Available!
              </h3>
              <p className="mb-2">
                The current provisional winner has declared <strong>{currentWinnerValue} ₿</strong>. 
                Your encrypted bid is <strong className="text-white bg-purple-900/50 px-1 rounded">{secret?.value} ₿</strong>.
              </p>
              <p className="text-purple-300/80">Click the button below to generate the proof and take the lead.</p>
            </div>
          ) : secret ? (
            <div className="bg-green-500/10 border border-green-500/20 rounded-lg p-3 text-sm text-green-300">
              Secret retrieved from secure storage. Value: <strong>{secret.value}</strong>
            </div>
          ) : (
            <>
              <div className="bg-orange-500/10 border border-orange-500/20 rounded-lg p-3 text-sm text-orange-300">
                Secret not found in storage. Enter cryptographic data manually.
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
            <button type="submit" className={isChallenge ? "btn-primary bg-purple-600 hover:bg-purple-500 flex-1 justify-center border-purple-500" : "btn-primary flex-1 justify-center"} disabled={loading}>
              {loading ? "Generating Proof..." : isChallenge ? "🚀 Launch Challenge" : "Submit Reveal"}
            </button>
            <button type="button" className="btn-secondary" onClick={onClose}>Cancel</button>
          </div>
        </form>
      )}
    </Modal>
  );
}