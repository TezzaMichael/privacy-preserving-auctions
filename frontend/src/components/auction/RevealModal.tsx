"use client";
import { useState, useEffect } from "react";
import { toast } from "sonner";
import { api } from "@/lib/api";
import { useAuthStore } from "@/store/auth";
import { loadSecret } from "@/lib/crypto";
import type { SealedBid, BidSecret } from "@/types";
import Modal from "@/components/ui/Modal";

interface Props { auctionId: string; myBid: SealedBid; onClose: () => void; onSuccess: () => void; }

export default function RevealModal({ auctionId, myBid, onClose, onSuccess }: Props) {
  const { token } = useAuthStore();
  const [secret, setSecret] = useState<BidSecret | null>(null);
  const [manualValue, setManualValue] = useState("");
  const [manualBlinding, setManualBlinding] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => { setSecret(loadSecret(auctionId)); }, [auctionId]);

  async function handleReveal(e: React.FormEvent) {
    e.preventDefault();
    if (!token) return;
    setLoading(true);
    try {
      const value = secret ? secret.value : parseInt(manualValue);
      const blinding = secret ? secret.blinding_hex : manualBlinding;
      const proofPlaceholder = JSON.stringify({
        commitment: myBid.commitment_hex,
        nonce_commit: myBid.commitment_hex,
        s_value: blinding.slice(0, 64),
        s_blinding: blinding.slice(0, 64),
        revealed_value: value,
      });
      await api.proofs.revealWinner(token, auctionId, myBid.bid_id, value, proofPlaceholder);
      toast.success("Winner reveal submitted");
      onSuccess();
    } catch (err: any) {
      toast.error(err.message);
    } finally {
      setLoading(false);
    }
  }

  return (
    <Modal title="Reveal Winner Bid" onClose={onClose}>
      <form onSubmit={handleReveal} className="space-y-4">
        {secret ? (
          <div className="bg-green-500/10 border border-green-500/20 rounded-lg p-3 text-sm text-green-300">
            Secret found in session storage. Value: <strong>{secret.value}</strong>
          </div>
        ) : (
          <>
            <div className="bg-orange-500/10 border border-orange-500/20 rounded-lg p-3 text-sm text-orange-300">
              Secret not found in storage. Enter manually.
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
        <div className="flex gap-3">
          <button type="submit" className="btn-primary flex-1 justify-center" disabled={loading}>
            {loading ? "Submitting…" : "Submit Reveal"}
          </button>
          <button type="button" className="btn-secondary" onClick={onClose}>Cancel</button>
        </div>
      </form>
    </Modal>
  );
}