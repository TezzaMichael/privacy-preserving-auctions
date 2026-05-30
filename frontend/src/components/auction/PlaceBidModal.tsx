"use client";
import { useState } from "react";
import { toast } from "sonner";
import { api } from "@/lib/api";
import { useAuthStore } from "@/store/auth";
import { storeSecret, signCommitment } from "@/lib/crypto";
import { createCommitmentWasm } from "@/lib/wasm"; 
import type { Auction } from "@/types";
import Modal from "@/components/ui/Modal";

interface Props { 
  auction: Auction;
  onClose: () => void; 
  onSuccess: () => void; 
}

export default function PlaceBidModal({ auction, onClose, onSuccess }: Props) {
  const { token, user, secretKeyHex } = useAuthStore();
  const [value, setValue] = useState("");
  const [loading, setLoading] = useState(false);
  
  const auctionId = auction.id;
  const hasCryptoKey = !!secretKeyHex;

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    
    if (!token || !secretKeyHex) {
      toast.error("Private key not found in storage. Please logout and enter your password again to restore the cryptographic session.");
      return;
    }
    
    const bidValue = parseInt(value);
    
    if (isNaN(bidValue) || bidValue < auction.min_bid) {
      toast.error(`Minimum bid is ${auction.min_bid}`);
      return;
    }
    if (auction.max_bid !== null && bidValue > auction.max_bid) {
      toast.error(`Maximum bid is ${auction.max_bid}`);
      return;
    }
    if ((bidValue - auction.min_bid) % auction.bid_step !== 0) {
      toast.error(`Invalid bid. You must respect the step increment of ${auction.bid_step}.`);
      return;
    }

    setLoading(true);
    try {
      const blinding = Array.from(crypto.getRandomValues(new Uint8Array(32)))
        .map(b => b.toString(16).padStart(2, "0")).join("");
      
      const commitmentHex = await createCommitmentWasm(bidValue, blinding);
      
      if (!commitmentHex) {
        throw new Error("Unable to generate cryptographic commitment via WASM.");
      }
     
      const sigHex = await signCommitment(
        secretKeyHex,
        auctionId,
        commitmentHex
      );
      
      const secret = { auction_id: auctionId, value: bidValue, blinding_hex: blinding, commitment_hex: commitmentHex };
      storeSecret(secret);
      
      await api.bids.submit(token, auctionId, commitmentHex, sigHex);
      
      toast.success("Bid submitted successfully!");
      onSuccess();
    } catch (err: any) {
      toast.error(err.message);
    } finally {
      setLoading(false);
    }
  }

  return (
    <Modal title="Place Sealed Bid" onClose={onClose}>
      {!hasCryptoKey ? (
        <div className="bg-yellow-500/10 border-l-4 border-yellow-500 p-4 mb-4 text-sm text-yellow-300 rounded">
          <p className="font-bold text-yellow-400 mb-1">Cryptographic Session Expired</p>
          <p className="mb-2">
            You reloaded the page. For security reasons, your temporary private key was removed from the browser's memory.
          </p>
          <p className="font-semibold">
            Please close this window, logout, and login again to restore signing capabilities.
          </p>
          <button type="button" className="btn-secondary w-full mt-4" onClick={onClose}>
            Close
          </button>
        </div>
      ) : (
        <>
          <div className="bg-yellow-500/10 border border-yellow-500/20 rounded-lg p-3 mb-4 text-sm text-yellow-300">
            Your bid value is sealed with a Pedersen commitment. The server only sees the commitment — not your value.
          </div>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="block text-sm text-slate-400 mb-1">Bid Value</label>
              <input 
                className="input" 
                type="number" 
                min={auction.min_bid} 
                step={auction.bid_step} 
                max={auction.max_bid || undefined} 
                value={value} 
                onChange={e => setValue(e.target.value)} 
                required 
                placeholder="Enter your bid amount" 
              />
            </div>
            <div className="flex gap-3">
              <button type="submit" className="btn-primary flex-1 justify-center" disabled={loading}>
                {loading ? "Submitting..." : "Submit Sealed Bid"}
              </button>
              <button type="button" className="btn-secondary" onClick={onClose}>Cancel</button>
            </div>
          </form>
        </>
      )}
    </Modal>
  );
}