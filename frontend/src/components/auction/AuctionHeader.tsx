import Link from "next/link";
import { Play, Square, CheckCheck, Eye, FileCheck } from "lucide-react";
import type { Auction, SealedBid } from "@/types";
import { cn, statusColor, statusLabel, formatDate } from "@/lib/utils";

interface Props {
  auction: Auction;
  isCreator: boolean;
  myBid?: SealedBid;
  onTransition: (a: "open" | "close" | "finalize") => void;
  onBid: () => void;
  onReveal?: () => void;
  onLoserProof?: () => void;
  onVerifyClick?: () => void; 
}

// 1. CORREZIONE: aggiunto onVerifyClick qui sotto
export default function AuctionHeader({ auction, isCreator, myBid, onTransition, onBid, onReveal, onLoserProof, onVerifyClick }: Props) {
  const isAuctionEnded = new Date() >= new Date(auction.end_time);
  const canVerify = auction.status !== "Pending" && auction.status !== "BiddingOpen";

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
          {!isCreator && (
            canVerify ? (
              <Link href={`/auctions/${auction.id}/verify`} className="btn-secondary">
                <Eye size={14} /> Verify
              </Link>
            ) : (
              <button 
                className="btn-secondary opacity-50 cursor-not-allowed" 
                onClick={onVerifyClick}
              >
                <Eye size={14} /> Verify (Locked)
              </button>
            )
          )}
          
          {/* 2. CORREZIONE: Ho rimosso il Link "Verify" duplicato che si trovava qui */}
          
          {isCreator && auction.status === "Pending" && (
            <button className="btn-primary" onClick={() => onTransition("open")}><Play size={14} /> Open Bidding</button>
          )}
          
          {/* TIME-LOCK visivo applicato qui */}
          {isCreator && auction.status === "BiddingOpen" && !isAuctionEnded && (
              <span className="text-xs text-slate-500 italic px-2">Auto-closes at end time</span>
          )}
          
          {isCreator && auction.status === "ProofPhase" && (
            <button className="btn-primary" onClick={() => onTransition("finalize")}><CheckCheck size={14} /> Finalize</button>
          )}
          
          {!isCreator && auction.status === "BiddingOpen" && !myBid && !isAuctionEnded && (
            <button className="btn-primary" onClick={onBid}>Place Bid</button>
          )}
          
          {/* I bottoni ora appaiono SOLO se page.tsx decide di passarli */}
          {onReveal && (
            <button className="btn-primary" onClick={onReveal}><FileCheck size={14} /> Reveal Bid</button>
          )}
          
          {onLoserProof && (
            <button className="btn-secondary" onClick={onLoserProof}><FileCheck size={14} /> Submit Proof</button>
          )}
        </div>
      </div>
    </div>
  );
}