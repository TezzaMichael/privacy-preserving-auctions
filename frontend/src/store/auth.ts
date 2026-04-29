import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { User } from "@/types";

interface AuthState {
  token: string | null;
  user: User | null;
  secretKeyHex: string | null;
  setAuth: (token: string, user: User, secretKeyHex: string) => void;
  clearAuth: () => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    set => ({
      token: null,
      user: null,
      secretKeyHex: null,
      setAuth: (token, user, secretKeyHex) => set({ token, user, secretKeyHex }),
      clearAuth: () => set({ token: null, user: null, secretKeyHex: null }),
    }),
    { name: "auction-auth" }
  )
);