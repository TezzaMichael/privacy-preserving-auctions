"use client";
import { useState } from "react";
import { useRouter } from "next/navigation";
import { toast } from "sonner";
import { api } from "@/lib/api";
import { useAuthStore } from "@/store/auth";
import { useAuctionStore } from "@/store/auctions";

const getDefaultEndDate = () => {
  const d = new Date();
  d.setDate(d.getDate() + 1); 
  return new Date(d.getTime() - d.getTimezoneOffset() * 60000).toISOString().slice(0, 16);
};

export default function CreateAuctionPage() {
  const router = useRouter();
  const { token } = useAuthStore();
  const upsert = useAuctionStore(s => s.upsert);
  
  const [form, setForm] = useState({ 
    title: "", 
    description: "", 
    min_bid: "1", 
    max_bid: "", 
    step: "1", 
    end_time: getDefaultEndDate() 
  });
  
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!token) return;

    const minBid = parseInt(form.min_bid);
    const maxBid = parseInt(form.max_bid);  
    const step = parseInt(form.step);

    if (minBid >= maxBid) {
      toast.error("maxBid must be greater than minBid");
      return;
    }

    const end = new Date(form.end_time);
    const now = new Date();
    const duration = Math.floor((end.getTime() - now.getTime()) / 1000);

    if (duration < 60) {
      toast.error("End time must be at least 1 minute in the future");
      return;
    }

    setLoading(true);
    try {
      const r = await api.auctions.create(token, form.title, form.description, minBid, maxBid, step, duration);
      upsert(r.data);
      toast.success("Auction created successfully");
      router.push(`/auctions/${r.data.id}`);
    } catch (err: any) {
      toast.error(err.message);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="max-w-2xl mx-auto">
      <h1 className="text-3xl font-bold mb-8">Create Auction</h1>
      <div className="card">
        <form onSubmit={handleSubmit} className="space-y-5">
          <div>
            <label className="block text-sm text-slate-400 mb-1">Title</label>
            <input className="input" value={form.title} onChange={e => setForm(f => ({ ...f, title: e.target.value }))} required maxLength={120} />
          </div>
          <div>
            <label className="block text-sm text-slate-400 mb-1">Description</label>
            <textarea className="input resize-none" rows={4} value={form.description} onChange={e => setForm(f => ({ ...f, description: e.target.value }))} />
          </div>
          
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm text-slate-400 mb-1">Min Bid</label>
              <input className="input" type="number" min="1" required value={form.min_bid} onChange={e => setForm(f => ({ ...f, min_bid: e.target.value }))} />
            </div>
            <div>
              <label className="block text-sm text-slate-400 mb-1">Max Bid</label>
              <input className="input" type="number" min="2" required value={form.max_bid} onChange={e => setForm(f => ({ ...f, max_bid: e.target.value }))} placeholder="e.g. 1000" />
            </div>
            <div>
              <label className="block text-sm text-slate-400 mb-1">Bid Step</label>
              <input className="input" type="number" min="1" required value={form.step} onChange={e => setForm(f => ({ ...f, step: e.target.value }))} />
            </div>
            <div>
              <label className="block text-sm text-slate-400 mb-1">End Time</label>
              <input className="input" type="datetime-local" required value={form.end_time} onChange={e => setForm(f => ({ ...f, end_time: e.target.value }))} />
            </div>
          </div>

          <div className="flex gap-3 pt-2">
            <button type="submit" className="btn-primary" disabled={loading}>
              {loading ? "Creating..." : "Create Auction"}
            </button>
            <button type="button" className="btn-secondary" onClick={() => router.back()}>Cancel</button>
          </div>
        </form>
      </div>
    </div>
  );
}