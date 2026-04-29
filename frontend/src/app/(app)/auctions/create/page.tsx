"use client";
import { useState } from "react";
import { useRouter } from "next/navigation";
import { toast } from "sonner";
import { api } from "@/lib/api";
import { useAuthStore } from "@/store/auth";
import { useAuctionStore } from "@/store/auctions";

export default function CreateAuctionPage() {
  const router = useRouter();
  const { token } = useAuthStore();
  const upsert = useAuctionStore(s => s.upsert);
  const [form, setForm] = useState({ title: "", description: "", reserve_price: "" });
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!token) return;
    setLoading(true);
    try {
      const reserve = form.reserve_price ? parseInt(form.reserve_price) : undefined;
      const r = await api.auctions.create(token, form.title, form.description, reserve);
      upsert(r.data);
      toast.success("Auction created");
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
          <div>
            <label className="block text-sm text-slate-400 mb-1">Reserve Price (optional)</label>
            <input className="input" type="number" min="0" value={form.reserve_price} onChange={e => setForm(f => ({ ...f, reserve_price: e.target.value }))} placeholder="No reserve" />
          </div>
          <div className="flex gap-3 pt-2">
            <button type="submit" className="btn-primary" disabled={loading}>
              {loading ? "Creating…" : "Create Auction"}
            </button>
            <button type="button" className="btn-secondary" onClick={() => router.back()}>Cancel</button>
          </div>
        </form>
      </div>
    </div>
  );
}