import { useEffect, useState } from "react";
import Link from "next/link";
import { Play, Eye, FileCheck, Clock, Unlock, ShieldAlert } from "lucide-react";
import type { Auction, SealedBid } from "@/types";
import { cn, statusColor, statusLabel, formatDate } from "@/lib/utils";

interface Props {
  auction: Auction;
  isCreator: boolean;
  myBid?: SealedBid;
  mySecretValue?: number | null; 
  onTransition: (a: "open" | "close" | "finalize") => void;
  onBid: () => void;
  onReveal?: () => void; 
  onLoserProof?: () => void;
  isZkpChallenge?: boolean | null;
  hasWinner?: boolean;
}

export default function AuctionHeader({ 
  auction, 
  isCreator, 
  myBid, 
  mySecretValue, 
  onTransition, 
  onBid, 
  onReveal,      
  onLoserProof,
  isZkpChallenge,
  hasWinner 
}: Props) {
  const isAuctionEnded = new Date() >= new Date(auction.end_time);
  
  // Può verificare normalmente se c'è un vincitore o l'asta è in proof phase
  const canVerifyNormal = auction.status === "ProofPhase" || (auction.status === "Closed" && hasWinner);
  
  // Il bottone viene disattivato (invece che nascosto) se l'asta è chiusa ma senza vincitore
  const isVerifyDisabled = auction.status === "Closed" && !hasWinner;

  const [timeRemainingStr, setTimeRemainingStr] = useState<string>("");

  useEffect(() => {
    const updateCountdown = () => {
      const now = Date.now();
      const endTimeMs = new Date(auction.end_time).getTime();

      if (auction.status === "BiddingOpen") {
        const diff = endTimeMs - now;
        if (diff <= 0) {
          setTimeRemainingStr("Expiring...");
        } else {
          const mins = Math.floor(diff / 60000);
          const secs = Math.floor((diff % 60000) / 1000);
          setTimeRemainingStr(`Bidding: ${mins}m ${secs}s`);
        }
      } else {
        setTimeRemainingStr(""); 
      }
    };

    updateCountdown();
    const interval = setInterval(updateCountdown, 1000);
    return () => clearInterval(interval);
  }, [auction]);

  return (
    <div className="card">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="flex-1">
          <div className="flex items-center gap-3 mb-2">
            <span className={cn("badge", statusColor(auction.status))}>{statusLabel(auction.status)}</span>
          </div>
          <h1 className="text-2xl font-bold mb-1">{auction.title}</h1>
          <p className="text-slate-400 text-sm mb-4">{auction.description}</p>
          
          <div className="flex flex-wrap gap-4 mb-3 p-3 bg-surface border border-surface-border rounded-lg text-sm text-slate-300 w-fit">
            <div>Min Bid: <span className="mono text-brand-light">{auction.min_bid}</span></div>
            <div>Max Bid: <span className="mono text-brand-light">{auction.max_bid ?? "No limit"}</span></div>
            <div>Step: <span className="mono text-brand-light">{auction.bid_step}</span></div>
            <div>Ends: <span className="mono text-brand-light">{formatDate(auction.end_time)}</span></div>
          </div>
          
          <p className="mono text-xs text-slate-500">ID: {auction.id}</p>
        </div>
        
        <div className="flex flex-wrap gap-2 items-center">
          
          {mySecretValue !== undefined && mySecretValue !== null && myBid && (
            <div className="bg-slate-800 border border-blue-500/30 px-3 py-1.5 rounded-lg flex items-center gap-2 mr-2">
              <span className="text-xs text-slate-400 uppercase tracking-wider">Your Bid:</span>
              <span className="font-mono font-bold text-blue-400">{mySecretValue} ₿</span>
            </div>
          )}

          {timeRemainingStr && (
            <div className="text-xs font-mono bg-slate-800/80 border border-slate-700 px-3 py-1.5 rounded-lg text-amber-400 font-semibold flex items-center gap-1.5 animate-pulse">
              <Clock size={12} className="text-amber-500" />
              {timeRemainingStr}
            </div>
          )}

          {/* FIX: Gestione Intelligente del Bottone Verify */}
          {canVerifyNormal ? (
            <Link href={`/auctions/${auction.id}/verify`} className="btn-secondary">
              <Eye size={14} /> Verify Publicly
            </Link>
          ) : isVerifyDisabled ? (
            <button disabled className="btn-secondary opacity-40 cursor-not-allowed" title="No winner data to verify">
              <Eye size={14} /> Verify Publicly
            </button>
          ) : null}
          
          {isCreator && auction.status === "Pending" && (
            <button className="btn-primary" onClick={() => onTransition("open")}><Play size={14} /> Open Bidding</button>
          )}
          
          {!isCreator && auction.status === "BiddingOpen" && !myBid && !isAuctionEnded && (
            <button className="btn-primary" onClick={onBid}>Place Bid</button>
          )}

          {onReveal && (
            <button className="btn-primary flex items-center gap-1.5" onClick={onReveal}>
              <Unlock size={14} /> Reveal Bid
            </button>
          )}
          
          {onLoserProof && (
            <button 
              className={cn(
                "transition-all duration-200 flex items-center gap-1.5", 
                isZkpChallenge ? "btn-primary bg-purple-600 hover:bg-purple-500 border-purple-500 shadow-lg shadow-purple-900/20 text-white" : "btn-secondary"
              )} 
              onClick={onLoserProof}
            >
              {isZkpChallenge ? <><ShieldAlert size={14} /> Silent Challenge</> : <><FileCheck size={14} /> Submit Proof</>}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}