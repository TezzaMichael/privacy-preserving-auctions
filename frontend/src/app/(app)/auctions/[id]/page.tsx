"use client";
import { useEffect, useState } from "react";
import { useParams } from "next/navigation";
import { toast } from "sonner";
import { api } from "@/lib/api";
import type { Auction, SealedBid, WinnerReveal, LoserProof, BBEntry } from "@/types";
import { useAuthStore } from "@/store/auth";
import AuctionHeader from "@/components/auction/AuctionHeader";
import BidsPanel from "@/components/auction/BidsPanel";
import BulletinBoardPanel from "@/components/auction/BulletinBoardPanel";
import ProofsPanel from "@/components/auction/ProofsPanel";
import PlaceBidModal from "@/components/auction/PlaceBidModal";
import RevealModal from "@/components/auction/RevealModal";
import LoserProofModal from "@/components/auction/LoserProofModal";

export default function AuctionPage() {
  const { id } = useParams<{ id: string }>();
  const { token, user } = useAuthStore();
  const [auction, setAuction] = useState<Auction | null>(null);
  const [bids, setBids] = useState<SealedBid[]>([]);
  const [winner, setWinner] = useState<WinnerReveal | null>(null);
  const [losers, setLosers] = useState<LoserProof[]>([]);
  const [entries, setEntries] = useState<BBEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [showBidModal, setShowBidModal] = useState(false);
  const [showRevealModal, setShowRevealModal] = useState(false);
  const [showLoserModal, setShowLoserModal] = useState(false);

  async function load() {
    try {
      const [aR, bR, bbR] = await Promise.all([
        api.auctions.get(id),
        api.bids.list(id),
        api.board.get(id),
      ]);
      setAuction(aR.data);
      setBids(bR.data.bids);
      setEntries(bbR.data.entries);
      if (["ClaimPhase", "ProofPhase", "Closed"].includes(aR.data.status)) {
        try { const wR = await api.proofs.getWinner(id); setWinner(wR.data); } catch {}
      }
      if (["ProofPhase", "Closed"].includes(aR.data.status)) {
        try { const lR = await api.proofs.listLosers(id); setLosers(lR.data.proofs); } catch {}
      }
    } catch (err: any) {
      toast.error(err.message);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => { load(); }, [id]);

  async function transition(action: "open" | "close" | "finalize") {
    if (!token) return;
    try {
      const r = action === "open" ? await api.auctions.open(token, id)
        : action === "close" ? await api.auctions.close(token, id)
        : await api.auctions.finalize(token, id);
      setAuction(r.data);
      toast.success(`Auction ${action}ed`);
      load();
    } catch (err: any) {
      toast.error(err.message);
    }
  }

  if (loading) return <div className="animate-pulse space-y-4"><div className="card h-32" /><div className="card h-64" /></div>;
  if (!auction) return <div className="text-center py-24 text-slate-500">Auction not found</div>;

  const isCreator = user?.user_id === auction.creator_id;
  const myBid = bids.find(b => b.bidder_id === user?.user_id);

  return (
    <div className="space-y-6">
      <AuctionHeader
        auction={auction}
        isCreator={isCreator}
        myBid={myBid}
        onTransition={transition}
        onBid={() => setShowBidModal(true)}
        onReveal={() => setShowRevealModal(true)}
        onLoserProof={() => setShowLoserModal(true)}
      />
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <BidsPanel bids={bids} currentUserId={user?.user_id} />
        <ProofsPanel winner={winner} losers={losers} bids={bids} />
      </div>
      <BulletinBoardPanel entries={entries} onRefresh={load} />
      {showBidModal && (
        <PlaceBidModal auctionId={id} onClose={() => setShowBidModal(false)} onSuccess={() => { setShowBidModal(false); load(); }} />
      )}
      {showRevealModal && myBid && (
        <RevealModal auctionId={id} myBid={myBid} onClose={() => setShowRevealModal(false)} onSuccess={() => { setShowRevealModal(false); load(); }} />
      )}
      {showLoserModal && myBid && (
        <LoserProofModal auctionId={id} myBid={myBid} winnerValue={winner?.revealed_value ?? 0} onClose={() => setShowLoserModal(false)} onSuccess={() => { setShowLoserModal(false); load(); }} />
      )}
    </div>
  );
}