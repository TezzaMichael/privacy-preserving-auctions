"use client";
import { useEffect, useState } from "react";
import Link from "next/link";
import { Plus, RefreshCw } from "lucide-react";
import { useAuctionStore } from "@/store/auctions";
import AuctionCard from "@/components/AuctionCard";

export default function DashboardPage() {
  const { auctions, loading, fetch } = useAuctionStore();
  // Forza un re-render periodico per ricalcolare gli stati derivati (i timer che scadono)
  const [tick, setTick] = useState(0);

  useEffect(() => { 
    fetch(); 
  }, [fetch]);

  // Aggiorna l'orologio interno ogni 5 secondi per far "scadere" le aste in tempo reale sulla dashboard
  useEffect(() => {
    const interval = setInterval(() => setTick(t => t + 1), 5000);
    return () => clearInterval(interval);
  }, []);

  // 1. Calcola lo "Stato Derivato": se il radar è palesemente scaduto, forza lo stato su Closed.
  // 2. Applica il filtro delle 24 ore.
  const activeAuctions = auctions.map(a => {
    let derivedStatus = a.status;

    // Se il server dice che è in ClaimPhase, verifichiamo matematicamente se il tempo è scaduto
    if (derivedStatus === "ClaimPhase" && a.max_bid !== null) {
      const warmUpMs = 60000;
      const max = a.max_bid;
      const min = a.min_bid || 0;
      const step = a.bid_step || 1;
      const secondsPerStep = 2;
      
      const maxPollingMs = ((max - min) / step) * secondsPerStep * 1000;
      // Momento esatto in cui il radar tocca il valore minimo (min_bid)
      const limitHitTimeMs = new Date(a.end_time).getTime() + warmUpMs + maxPollingMs;

      // Se il momento attuale ha superato la fine del radar + i 3 secondi di tolleranza, considerala chiusa
      if (Date.now() > limitHitTimeMs + 3000) {
        derivedStatus = "Closed";
      }
    }

    return { ...a, status: derivedStatus };
  }).filter(a => {
    if (a.status !== "Closed") return true; 
    
    // Per le aste chiuse, calcola quanto tempo è passato dalla fine ufficiale
    const msSinceEnd = Date.now() - new Date(a.end_time).getTime();
    const hoursSinceEnd = msSinceEnd / (1000 * 60 * 60);
    
    // Mostra le aste chiuse solo se sono passate meno di 24 ore
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
          {/* Usiamo le aste con lo status "corretto" in locale */}
          {activeAuctions.map(a => <AuctionCard key={a.id} auction={a} />)}
        </div>
      )}
    </div>
  );
}