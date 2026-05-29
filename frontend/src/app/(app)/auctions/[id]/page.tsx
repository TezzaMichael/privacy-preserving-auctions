"use client";
import { useEffect, useState, useRef } from "react";
import { useParams } from "next/navigation";
import { toast } from "sonner";
import { api } from "@/lib/api";
import type { Auction, SealedBid, WinnerReveal, LoserProof, BBEntry } from "@/types";
import { useAuthStore } from "@/store/auth";
import { loadSecret } from "@/lib/crypto";
import { createProofOfOpeningWasm } from "@/lib/wasm";

import AuctionHeader from "@/components/auction/AuctionHeader";
import BidsPanel from "@/components/auction/BidsPanel";
import BulletinBoardPanel from "@/components/auction/BulletinBoardPanel";
import ProofsPanel from "@/components/auction/ProofsPanel";
import PlaceBidModal from "@/components/auction/PlaceBidModal";
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
  const [showLoserModal, setShowLoserModal] = useState(false);

  // Stati del Radar
  const [currentPollingPrice, setCurrentPollingPrice] = useState<number | null>(null);
  const [countdownRemaining, setCountdownRemaining] = useState<number | null>(null);
  const [pollingFailed, setPollingFailed] = useState(false);
  const autoRevealTimerRef = useRef<NodeJS.Timeout | null>(null);
  const isClaimingRef = useRef(false);
  const [mySecretValue, setMySecretValue] = useState<number | null>(null);

  async function load() {
    try {
      const [aR, bR, bbR] = await Promise.all([
        api.auctions.get(id), api.bids.list(id), api.board.get(id),
      ]);
      setAuction(aR.data);
      setBids(bR.data.bids);
      setEntries(bbR.data.entries);
      
      const secret = loadSecret(id);
      if (secret) {
        setMySecretValue(secret.value);
      }

      if (["ClaimPhase", "ProofPhase", "Closed"].includes(aR.data.status)) {
        try { const wR = await api.proofs.getWinner(id); setWinner(wR.data); } catch {}
      }
      if (["ProofPhase", "Closed"].includes(aR.data.status)) {
        try { const lR = await api.proofs.listLosers(id); setLosers(lR.data.proofs); } catch {}
      }
    } catch (err: any) { toast.error(err.message); } finally { setLoading(false); }
  }

  useEffect(() => { 
    load();
    const secret = loadSecret(id);
    if (secret) {
      setMySecretValue(secret.value);
    }
  }, [id]);

  // SINCRONIZZAZIONE SILENZIOSA
  useEffect(() => {
    if (!auction || auction.status === "Closed") return;
    const interval = setInterval(() => {
      api.auctions.get(id).then(aR => {
        if (aR.data.status !== auction.status) {
          load();
        } else if (aR.data.status === "ClaimPhase" || aR.data.status === "ProofPhase") {
          api.proofs.getWinner(id).then(wR => { if (wR.data) setWinner(wR.data); }).catch(() => {});
          if (aR.data.status === "ProofPhase") {
            api.proofs.listLosers(id).then(lR => setLosers(lR.data.proofs)).catch(() => {});
          }
        }
      }).catch(() => {});
    }, 1500);
    return () => clearInterval(interval);
  }, [auction?.status, id]);

  // EFFETTO RADAR CON WARM-UP DI 2 MINUTI
  useEffect(() => {
    if (!auction || auction.status !== "ClaimPhase" || !auction.max_bid || winner) return;
    
    const maxBid = auction.max_bid;
    const bidStep = auction.bid_step;
    const limit = auction.min_bid || 0;
    const secondsPerStep = 2;
    const warmUpMs = 60 * 1000;
    const start = new Date(auction.end_time).getTime();

    const updatePrice = () => {
      if (isClaimingRef.current) return;

      const now = Date.now();
      const elapsedMs = now - start;

      if (elapsedMs < warmUpMs) {
        const remainingSecs = Math.ceil((warmUpMs - elapsedMs) / 1000);
        setCountdownRemaining(remainingSecs > 0 ? remainingSecs : 0);
        setCurrentPollingPrice(maxBid);
      } else {
        setCountdownRemaining(null);
        const elapsedSecsAfterWarmup = Math.floor((elapsedMs - warmUpMs) / 1000);
        const steps = Math.floor(elapsedSecsAfterWarmup / secondsPerStep);
        let current = maxBid - (steps * bidStep);
        
        if (current <= limit) {
          setCurrentPollingPrice(limit);
          setPollingFailed(true);
          // Invio chiusura forzata se il creatore sta osservando
          if (user?.user_id === auction.creator_id) {
             api.auctions.finalize(token!, id).then(() => load()).catch(() => {});
          }
        } else {
          setCurrentPollingPrice(current);
        }
      }
    };

    updatePrice(); 
    const interval = setInterval(updatePrice, 1000);
    return () => clearInterval(interval);
  }, [auction?.status, winner, id]);

  // AUTOMAZIONE INVISIBILE CALIBRATA
  const myBid = bids.find(b => b.bidder_id === user?.user_id);
  useEffect(() => {
    if (!auction || auction.status !== "ClaimPhase" || !myBid || !token || winner) return;
    
    const storedBidData = loadSecret(id); 
    if (storedBidData) {
        try {
            const myValue = storedBidData.value;
            const maxBid = auction.max_bid || 0;
            const bidStep = auction.bid_step;
            const secondsPerStep = 2;
            const warmUpMs = 60 * 1000;
            const claimStartTime = new Date(auction.end_time).getTime();
            const now = Date.now();
            
            const stepsToMyBid = (maxBid - myValue) / bidStep;
            const targetTimeMs = claimStartTime + warmUpMs + (stepsToMyBid * secondsPerStep * 1000);
            const msUntilMyTurn = targetTimeMs - now;

            if (autoRevealTimerRef.current) clearTimeout(autoRevealTimerRef.current);

            const executeAutoReveal = async () => {
                if (winner) return;
                isClaimingRef.current = true;
                try {
                    toast.loading(`Submitting claim for ${myValue} ₿...`, { id: "reveal-toast" });
                    const realProof = await createProofOfOpeningWasm(myValue, storedBidData.blinding_hex, myBid.commitment_hex);
                    if (!realProof) throw new Error("WASM Generation Failed");

                    await api.proofs.revealWinner(token, id, myBid.bid_id, myValue, realProof);
                    toast.success("Bid claimed successfully!", { id: "reveal-toast" });
                    load(); 
                } catch (err) {
                    toast.dismiss("reveal-toast");
                }
            };

            if (msUntilMyTurn > 0) {
                autoRevealTimerRef.current = setTimeout(executeAutoReveal, msUntilMyTurn);
            } else if (now >= claimStartTime + warmUpMs) {
                executeAutoReveal();
            }
        } catch (e) { console.error(e); }
    }
    return () => { if (autoRevealTimerRef.current) clearTimeout(autoRevealTimerRef.current); };
  }, [auction?.status, myBid, winner, id, token]);

  useEffect(() => {
    if (!auction || auction.status !== "ProofPhase" || !myBid || !token || !winner) return;
    
    if (mySecretValue !== null && mySecretValue > winner.revealed_value) {
        const executeChallenge = async () => {
            try {
                toast.loading("Higher value detected! Taking over the lead...", { id: "challenge" });
                const storedBidData = loadSecret(id);
                
                if (!storedBidData) throw new Error("Bid data lost from browser.");

                const realProof = await createProofOfOpeningWasm(mySecretValue, storedBidData.blinding_hex, myBid.commitment_hex);
                if (!realProof) throw new Error("Proof generation failed.");

                // RISOLUZIONE BUG: Si usa revealWinner (che fa l'override nel backend), non submitLoser!
                await api.proofs.revealWinner(token, id, myBid.bid_id, mySecretValue, realProof);
                
                toast.success("Challenge successful! You are the new provisional winner.", { id: "challenge" });
                load();
            } catch (err: any) {
                toast.error(err.message || "Failed to challenge", { id: "challenge" });
            }
        };
        executeChallenge();
    }
  }, [auction?.status, myBid, winner, mySecretValue, id, token]);

  async function transition(action: "open" | "close" | "finalize") {
    if (!token) return;
    try {
      const r = action === "open" ? await api.auctions.open(token, id) : await api.auctions.close(token, id);
      setAuction(r.data);
      load();
    } catch (err: any) { toast.error(err.message); }
  }

  if (loading) return <div className="animate-pulse space-y-4"><div className="card h-32" /><div className="card h-64" /></div>;
  if (!auction) return <div className="text-center py-24 text-slate-500">Auction not found</div>;

  const isCreator = user?.user_id === auction?.creator_id;
  const iAmLoserAndHaveProven = losers.some(l => l.bidder_id === user?.user_id);
  const hasValidWinner = winner && winner.winner_id !== "00000000-0000-0000-0000-000000000000";
  const amIWinner = hasValidWinner && winner.winner_id === user?.user_id;

  const visibleBids = bids.filter(b => 
    b.bidder_id === user?.user_id || 
    b.bidder_id === winner?.winner_id
  );

  return (
    <div className="space-y-6">
      <AuctionHeader
        auction={auction}
        isCreator={isCreator}
        mySecretValue={mySecretValue}
        myBid={myBid}
        hasWinner={!!winner}
        onTransition={transition}
        onBid={() => setShowBidModal(true)}
        onLoserProof={(
          auction.status === "ProofPhase" && 
          !isCreator && 
          myBid && 
          winner?.winner_id !== user?.user_id && 
          !iAmLoserAndHaveProven
        ) ? () => setShowLoserModal(true) : undefined}
      />
      
    

      {auction.status === "ProofPhase" && winner && (
        <div className={`border rounded-xl p-6 text-left shadow-lg transition-all duration-300 ${
          amIWinner 
            ? 'bg-emerald-950/40 border-emerald-500/50 shadow-emerald-900/20' 
            : 'bg-amber-950/40 border-amber-500/50 shadow-amber-900/20'
        }`}>
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
            <div>
              <h2 className={`text-lg font-bold uppercase tracking-wider mb-1 flex items-center gap-2 ${
                amIWinner ? 'text-emerald-400' : 'text-amber-400'
              }`}>
                {amIWinner ? '🎉 Provisional Winner' : '⚖️ Standing Update'}
              </h2>
              <p className="text-slate-200 text-sm">
                {amIWinner 
                  ? "You are currently the winner! Wait for other participants' proofs or until the phase closes."
                  : "You are not the winner currently. If your bid is higher, submit a proof to challenge!"
                }
              </p>
            </div>
            <div className="sm:text-right bg-slate-900/50 px-4 py-2 rounded-lg border border-white/5">
              <div className="text-xs text-slate-400 mb-1 uppercase tracking-wider">Current Winning Bid</div>
              <div className="text-2xl font-mono font-bold text-white">{winner.revealed_value} ₿</div>
            </div>
          </div>
        </div>
      )}

      {auction.status === "Closed" && (
        <div className={`border rounded-xl p-6 text-left shadow-lg transition-all duration-300 ${
          hasValidWinner 
            ? amIWinner 
              ? 'bg-emerald-950/40 border-emerald-500/50 shadow-emerald-900/20' 
              : 'bg-rose-950/40 border-rose-500/50 shadow-rose-900/20'
            : 'bg-slate-800 border-slate-600 shadow-slate-900/20'
        }`}>
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
            <div>
              <h2 className={`text-lg font-bold uppercase tracking-wider mb-1 flex items-center gap-2 ${
                hasValidWinner 
                  ? amIWinner ? 'text-emerald-400' : 'text-rose-400'
                  : 'text-slate-400'
              }`}>
                {hasValidWinner 
                  ? amIWinner ? '🏆 YOU WON!' : '❌ YOU LOST' 
                  : '⚪ NO WINNER'
                }
              </h2>
              <p className="text-slate-200 text-sm">
                {hasValidWinner 
                  ? amIWinner 
                    ? `Congratulations! You won the auction with a final bid of ${winner.revealed_value} ₿.`
                    : `The auction has ended. The winning bid was ${winner.revealed_value} ₿.`
                  : "The auction closed without any verified claims. There is no winner for this auction."
                }
              </p>
            </div>
            {hasValidWinner && (
              <div className="sm:text-right bg-slate-900/50 px-4 py-2 rounded-lg border border-white/5">
                <div className="text-xs text-slate-400 mb-1 uppercase tracking-wider">Final Price</div>
                <div className="text-2xl font-mono font-bold text-white">{winner.revealed_value} ₿</div>
              </div>
            )}
          </div>
        </div>
      )}

      {auction.status === "Closed" && !winner && (
        <div className="border rounded-xl p-8 text-center shadow-lg bg-slate-900 border-red-500/30 shadow-red-900/20">
          <h2 className="text-xl font-medium text-red-400 uppercase tracking-widest mb-2">Auction Closed</h2>
          <div className="text-4xl font-bold text-white mb-2">No Winner</div>
          <p className="text-slate-500 text-sm">No valid bids were claimed during the polling phase.</p>
        </div>
      )}
      
      {/* SCHERMO RADAR - MOSTRATO SOLO IN CLAIM PHASE E SOLO SE NON C'È ANCORA UN VINCITORE */}
      {auction.status === "ClaimPhase" && !winner && (
        <div className={`border rounded-xl p-8 text-center shadow-lg relative overflow-hidden transition-all duration-500 ${
          countdownRemaining !== null ? 'bg-slate-900/60 border-amber-500/30 shadow-amber-900/10' :
          pollingFailed ? 'bg-red-950/20 border-red-500/30 shadow-red-900/20' :
          'bg-slate-900 border-blue-500/30 shadow-blue-900/20'
        }`}>
          {countdownRemaining !== null ? (
            <>
              <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-transparent via-amber-500 to-transparent opacity-50"></div>
              <h2 className="text-xl font-medium text-amber-400 uppercase tracking-widest mb-2 flex items-center justify-center gap-2 animate-pulse">
                Polling Preparation
              </h2>
              <div className="text-5xl font-mono font-bold text-white tracking-tight my-4">
                Starts in: <span className="text-amber-400">{countdownRemaining}s</span>
              </div>
              <p className="text-xs text-slate-500 bg-surface border border-surface-border inline-block px-4 py-1 rounded-full">
                Starting price is locked at <strong>{auction.max_bid} ₿</strong>.
              </p>
            </>
          ) : pollingFailed ? (
            <>
              <h2 className="text-xl font-medium text-red-400 uppercase tracking-widest mb-2">Polling Ended</h2>
              <div className="text-4xl font-bold text-white mb-4">No Winner</div>
            </>
          ) : (
            <>
              <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-transparent via-blue-500 to-transparent opacity-50 animate-pulse"></div>
              <h2 className="text-xl font-medium text-slate-400 uppercase tracking-widest mb-2 flex items-center justify-center gap-2">
                <span className="w-2 h-2 rounded-full bg-blue-500 animate-ping"></span>
                Active Polling
              </h2>
              <div className="text-6xl font-mono font-bold text-white tracking-tight flex justify-center items-baseline gap-2">
                <span className="text-blue-500 text-4xl">₿</span> 
                {currentPollingPrice !== null ? currentPollingPrice : "---"}
              </div>
            </>
          )}
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <BidsPanel bids={visibleBids} currentUserId={user?.user_id} />
        <ProofsPanel winner={winner} losers={losers} bids={bids} />
      </div>
      <BulletinBoardPanel entries={entries} onRefresh={load} />
      
      {showBidModal && <PlaceBidModal auction={auction} onClose={() => setShowBidModal(false)} onSuccess={() => { setShowBidModal(false); load(); }} />}
      {showLoserModal && myBid && <LoserProofModal auctionId={id} myBid={myBid} winnerValue={winner?.revealed_value ?? 0} onClose={() => setShowLoserModal(false)} onSuccess={() => { setShowLoserModal(false); load(); }} />}
    </div>
  );
}