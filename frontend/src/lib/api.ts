import axios, { AxiosInstance } from "axios";
import type {
  Auction, SealedBid, BBEntry, WinnerReveal, LoserProof, ServerPublicKey, User,
} from "@/types";

const BASE = process.env.NEXT_PUBLIC_API_URL ?? "http://127.0.0.1:8080";

export function createApi(token?: string): AxiosInstance {
  const instance = axios.create({ baseURL: BASE });
  if (token) instance.defaults.headers.common["Authorization"] = `Bearer ${token}`;
  instance.interceptors.response.use(
    r => r,
    err => {
      const msg = err.response?.data?.error ?? err.message;
      return Promise.reject(new Error(msg));
    }
  );
  return instance;
}

export const api = {
  auth: {
    register: (username: string, password: string, public_key_hex: string) =>
      createApi().post<{ user_id: string; username: string; public_key_hex: string }>(
        "/auth/register", { username, password, public_key_hex }
      ),
    login: (username: string, password: string) =>
      createApi().post<{ jwt_token: string; user_id: string; username: string; public_key_hex: string }>(
        "/auth/login", { username, password }
      ),
    me: (token: string) =>
      createApi(token).get<User>("/auth/me"),
  },
  auctions: {
    list: () => createApi().get<{ auctions: Auction[]; total: number }>("/auctions"),
    get: (id: string) => createApi().get<Auction>(`/auctions/${id}`),
    create: (token: string, title: string, description: string, reserve_price?: number) =>
      createApi(token).post<Auction>("/auctions", { title, description, reserve_price }),
    open: (token: string, id: string) =>
      createApi(token).post<Auction>(`/auctions/${id}/open`, {}),
    close: (token: string, id: string) =>
      createApi(token).post<Auction>(`/auctions/${id}/close`, {}),
    finalize: (token: string, id: string) =>
      createApi(token).post<Auction>(`/auctions/${id}/finalize`, {}),
  },
  bids: {
    list: (auction_id: string) =>
      createApi().get<{ bids: SealedBid[]; total: number }>(`/auctions/${auction_id}/bids`),
    submit: (token: string, auction_id: string, commitment_hex: string, bidder_signature_hex: string) =>
      createApi(token).post<{ bid_id: string; bb_entry_hash_hex: string; bb_sequence: number }>(
        `/auctions/${auction_id}/bids`, { commitment_hex, bidder_signature_hex }
      ),
  },
  proofs: {
    revealWinner: (token: string, auction_id: string, bid_id: string, revealed_value: number, proof_json: string) =>
      createApi(token).post<WinnerReveal>(`/auctions/${auction_id}/reveal`, { bid_id, revealed_value, proof_json }),
    getWinner: (auction_id: string) =>
      createApi().get<WinnerReveal>(`/auctions/${auction_id}/reveal`),
    submitLoser: (token: string, auction_id: string, bid_id: string, revealed_value: number, proof_json: string) =>
      createApi(token).post<LoserProof>(`/auctions/${auction_id}/loser-proofs`, { bid_id, revealed_value, proof_json }),
    listLosers: (auction_id: string) =>
      createApi().get<{ proofs: LoserProof[]; total: number }>(`/auctions/${auction_id}/loser-proofs`),
  },
  board: {
    get: (auction_id: string) =>
      createApi().get<{ auction_id: string; entries: BBEntry[]; total: number; head_hash_hex: string }>(
        `/bulletin-board/${auction_id}`
      ),
    getEntry: (auction_id: string, sequence: number) =>
      createApi().get<{ entry: BBEntry }>(`/bulletin-board/${auction_id}/${sequence}`),
  },
  server: {
    publicKey: () => createApi().get<ServerPublicKey>("/server/public-key"),
  },
};