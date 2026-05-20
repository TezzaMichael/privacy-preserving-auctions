"use client";
import { useState, useEffect } from "react";
import { toast } from "sonner";
import { api } from "@/lib/api";
import { useAuthStore } from "@/store/auth";
import { loadSecret } from "@/lib/crypto";
import { createProofOfOpeningWasm } from "@/lib/wasm"; // <-- Aggiunto l'import
import type { SealedBid, BidSecret } from "@/types";
import Modal from "@/components/ui/Modal";

interface Props {
  auctionId: string; myBid: SealedBid; winnerValue: number;
  onClose: () => void; onSuccess: () => void;
}

export default function LoserProofModal({ auctionId, myBid, winnerValue, onClose, onSuccess }: Props) {
  const { token } = useAuthStore();
  const [secret, setSecret] = useState<BidSecret | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => { setSecret(loadSecret(auctionId)); }, [auctionId]);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!token || !secret) return;
    if (secret.value >= winnerValue) {
      toast.error("Your value is not less than the winner's");
      return;
    }
    setLoading(true);
    try {
      // Generazione del vero proof crittografico tramite WASM
      const realProof = await createProofOfOpeningWasm(secret.value, secret.blinding_hex, myBid.commitment_hex);
      if (!realProof) {
        throw new Error("Impossibile generare il proof di sottomissione tramite WASM.");
      }

      await api.proofs.submitLoser(token, auctionId, myBid.bid_id, secret.value, realProof);
      toast.success("Loser proof submitted");
      onSuccess();
    } catch (err: any) {
      toast.error(err.message);
    } finally {
      setLoading(false);
    }
  }

  return (
    <Modal title="Submit Loser Proof" onClose={onClose}>
      <form onSubmit={handleSubmit} className="space-y-4">
        {secret ? (
          <div className="space-y-2">
            <div className="bg-surface-border/50 rounded-lg p-3 text-sm">
              <div className="flex justify-between"><span className="text-slate-400">Your bid:</span><span className="font-semibold">{secret.value}</span></div>
              <div className="flex justify-between mt-1"><span className="text-slate-400">Winner bid:</span><span className="font-semibold text-yellow-400">{winnerValue}</span></div>
            </div>
            {secret.value >= winnerValue && (
              <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-3 text-sm text-red-400">
                Your bid is not less than the winner's bid.
              </div>
            )}
          </div>
        ) : (
          <div className="bg-orange-500/10 border border-orange-500/20 rounded-lg p-3 text-sm text-orange-300">
            Secret not found in session storage. Cannot generate loser proof.
          </div>
        )}
        <div className="flex gap-3">
          <button type="submit" className="btn-primary flex-1 justify-center" disabled={loading || !secret || (secret?.value ?? 0) >= winnerValue}>
            {loading ? "Submitting…" : "Submit Loser Proof"}
          </button>
          <button type="button" className="btn-secondary" onClick={onClose}>Cancel</button>
        </div>
      </form>
    </Modal>
  );
}