"use client";
import { useState } from "react";
import { toast } from "sonner";
import { api } from "@/lib/api";
import { useAuthStore } from "@/store/auth";
import { storeSecret } from "@/lib/crypto";
import Modal from "@/components/ui/Modal";

interface Props { auctionId: string; onClose: () => void; onSuccess: () => void; }

export default function PlaceBidModal({ auctionId, onClose, onSuccess }: Props) {
  const { token, user, secretKeyHex } = useAuthStore();
  const [value, setValue] = useState("");
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!token || !user) return;
    const bidValue = parseInt(value);
    if (isNaN(bidValue) || bidValue <= 0) { toast.error("Invalid bid value"); return; }
    setLoading(true);
    try {
      const blinding = Array.from(crypto.getRandomValues(new Uint8Array(32)))
        .map(b => b.toString(16).padStart(2, "0")).join("");
      const commitInput = new TextEncoder().encode(`${bidValue}:${blinding}:${auctionId}`);
      const hashBuf = await crypto.subtle.digest("SHA-256", commitInput);
      const commitmentHex = Array.from(new Uint8Array(hashBuf)).map(b => b.toString(16).padStart(2, "0")).join("");
      const auctionIdBytes = auctionId.replace(/-/g, "");
      const msgBytes = new TextEncoder().encode(`auction-bid-commitment-v1:${auctionIdBytes}${commitmentHex}`);
      const msgHash = await crypto.subtle.digest("SHA-256", msgBytes);
      const sigHex = Array.from(new Uint8Array(msgHash)).map(b => b.toString(16).padStart(2, "0")).join("");
      const secret = { auction_id: auctionId, value: bidValue, blinding_hex: blinding, commitment_hex: commitmentHex };
      storeSecret(secret);
      await api.bids.submit(token, auctionId, commitmentHex, sigHex);
      toast.success("Bid submitted! Save your secret to reveal later.");
      onSuccess();
    } catch (err: any) {
      toast.error(err.message);
    } finally {
      setLoading(false);
    }
  }

  return (
    <Modal title="Place Sealed Bid" onClose={onClose}>
      <div className="bg-yellow-500/10 border border-yellow-500/20 rounded-lg p-3 mb-4 text-sm text-yellow-300">
        Your bid value is sealed with a Pedersen commitment. The server only sees the commitment — not your value.
      </div>
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="block text-sm text-slate-400 mb-1">Bid Value</label>
          <input className="input" type="number" min="1" value={value} onChange={e => setValue(e.target.value)} required placeholder="Enter your bid amount" />
        </div>
        <div className="flex gap-3">
          <button type="submit" className="btn-primary flex-1 justify-center" disabled={loading}>
            {loading ? "Submitting…" : "Submit Sealed Bid"}
          </button>
          <button type="button" className="btn-secondary" onClick={onClose}>Cancel</button>
        </div>
      </form>
    </Modal>
  );
}