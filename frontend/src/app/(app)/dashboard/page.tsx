"use client";
import { useEffect, useState } from "react";
import Link from "next/link";
import { Plus, RefreshCw } from "lucide-react";
import { useAuctionStore } from "@/store/auctions";
import AuctionCard from "@/components/AuctionCard";

export default function DashboardPage() {
  const { auctions, loading, fetch } = useAuctionStore();
  const [tick, setTick] = useState(0);

  useEffect(() => { 
    fetch(); 
  }, [fetch]);

  useEffect(() => {
    const interval = setInterval(() => setTick(t => t + 1), 5000);
    return () => clearInterval(interval);
  }, []);

  const activeAuctions = auctions.map(a => {
    let derivedStatus = a.status;

    if (derivedStatus === "ClaimPhase" && a.max_bid !== null) {
      const warmUpMs = 60000;
      const max = a.max_bid;
      const min = a.min_bid || 0;
      const step = a.bid_step || 1;
      const secondsPerStep = 2;
      
      const maxPollingMs = ((max - min) / step) * secondsPerStep * 1000;
      const limitHitTimeMs = new Date(a.end_time).getTime() + warmUpMs + maxPollingMs;

      if (Date.now() > limitHitTimeMs + 3000) {
        derivedStatus = "Closed";
      }
    }

    return { ...a, status: derivedStatus };
  }).filter(a => {
    if (a.status !== "Closed") return true; 
    
    const msSinceEnd = Date.now() - new Date(a.end_time).getTime();
    const hoursSinceEnd = msSinceEnd / (1000 * 60 * 60);
    
    // Show auctions that ended less than 24 hours ago 
    return hoursSinceEnd < 24; 
  });

  return (
    <div>
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold">Auctions</h1>
          <p className="text-slate-400 mt-1">
            {activeAuctions.length} auction{activeAuctions.length !== 1 ? "s" : ""}
          </p>
        </div>
        <div className="flex gap-3">
          <button onClick={fetch} className="btn-secondary" disabled={loading}>
            <RefreshCw size={16} className={loading ? "animate-spin" : ""} />
            Refresh
          </button>
          <Link href="/auctions/create" className="btn-primary">
            <Plus size={16} /> New Auction
          </Link>
        </div>
      </div>
      
      {loading && auctions.length === 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {[...Array(6)].map((_, i) => (
            <div key={i} className="card animate-pulse h-48 bg-surface-card" />
          ))}
        </div>
      ) : activeAuctions.length === 0 ? (
        <div className="text-center py-24 text-slate-500">
          <p className="text-lg">No active auctions yet.</p>
          <Link href="/auctions/create" className="btn-primary mt-4 inline-flex">Create one</Link>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {activeAuctions.map(a => <AuctionCard key={a.id} auction={a} />)}
        </div>
      )}
    </div>
  );
}