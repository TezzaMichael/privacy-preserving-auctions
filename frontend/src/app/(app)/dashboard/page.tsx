"use client";
import { useEffect } from "react";
import Link from "next/link";
import { Plus, RefreshCw } from "lucide-react";
import { useAuctionStore } from "@/store/auctions";
import AuctionCard from "@/components/AuctionCard";

export default function DashboardPage() {
  const { auctions, loading, fetch } = useAuctionStore();
  useEffect(() => { fetch(); }, [fetch]);

  return (
    <div>
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold">Auctions</h1>
          <p className="text-slate-400 mt-1">{auctions.length} auction{auctions.length !== 1 ? "s" : ""}</p>
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
      ) : auctions.length === 0 ? (
        <div className="text-center py-24 text-slate-500">
          <p className="text-lg">No auctions yet.</p>
          <Link href="/auctions/create" className="btn-primary mt-4 inline-flex">Create one</Link>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {auctions.map(a => <AuctionCard key={a.id} auction={a} />)}
        </div>
      )}
    </div>
  );
}