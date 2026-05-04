import Link from "next/link";
import { ArrowRight } from "lucide-react";
import type { Auction } from "@/types";
import { cn, statusColor, statusLabel, formatDate } from "@/lib/utils";

export default function AuctionCard({ auction }: { auction: Auction }) {
  return (
    <Link href={`/auctions/${auction.id}`} className="card hover:border-brand/50 transition-colors group block">
      <div className="flex items-start justify-between mb-3">
        <span className={cn("badge", statusColor(auction.status))}>{statusLabel(auction.status)}</span>
        <ArrowRight size={16} className="text-slate-600 group-hover:text-brand transition-colors" />
      </div>
      <h3 className="font-semibold text-lg mb-1 line-clamp-1">{auction.title}</h3>
      <p className="text-slate-400 text-sm line-clamp-2 mb-4">{auction.description || "No description"}</p>
      <div className="flex items-center justify-between text-xs text-slate-500">
        <span>{formatDate(auction.created_at)}</span>
        <span className="text-slate-400 font-mono">Min: {auction.min_bid}</span>
      </div>
    </Link>
  );
}