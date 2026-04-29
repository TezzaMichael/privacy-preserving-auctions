import { create } from "zustand";
import type { Auction } from "@/types";
import { api } from "@/lib/api";

interface AuctionState {
  auctions: Auction[];
  loading: boolean;
  fetch: () => Promise<void>;
  upsert: (a: Auction) => void;
}

export const useAuctionStore = create<AuctionState>()(set => ({
  auctions: [],
  loading: false,
  fetch: async () => {
    set({ loading: true });
    try {
      const r = await api.auctions.list();
      set({ auctions: r.data.auctions });
    } finally {
      set({ loading: false });
    }
  },
  upsert: a => set(s => ({
    auctions: s.auctions.some(x => x.id === a.id)
      ? s.auctions.map(x => x.id === a.id ? a : x)
      : [a, ...s.auctions],
  })),
}));