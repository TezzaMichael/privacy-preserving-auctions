"use client";
import { useEffect, useState } from "react";
import { useParams } from "next/navigation";
import { api } from "@/lib/api";
import type { BBEntry, LoserProof, WinnerReveal, SealedBid, VerificationResult, ServerPublicKey, Auction } from "@/types";
import { verifyTranscriptWasm, wasmAvailable } from "@/lib/wasm";
import VerificationPanel from "@/components/verify/VerificationPanel";
import TranscriptViewer from "@/components/verify/TranscriptViewer";
import { toast } from "sonner";

export default function VerifyPage() {
  const { id } = useParams<{ id: string }>();
  
  const [auction, setAuction] = useState<Auction | null>(null);
  const [entries, setEntries] = useState<BBEntry[]>([]);
  const [bids, setBids] = useState<SealedBid[]>([]);
  const [winner, setWinner] = useState<WinnerReveal | null>(null);
  const [losers, setLosers] = useState<LoserProof[]>([]);
  const [serverKey, setServerKey] = useState<ServerPublicKey | null>(null);
  const [result, setResult] = useState<VerificationResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [hasWasm, setHasWasm] = useState(false);

  useEffect(() => {
    wasmAvailable().then(setHasWasm);
    async function load() {
      try {
        const [aucR, bbR, bR, skR] = await Promise.all([
          api.auctions.get(id),
          api.board.get(id), 
          api.bids.list(id), 
          api.server.publicKey(),
        ]);
        setAuction(aucR.data);
        setEntries(bbR.data.entries);
        setBids(bR.data.bids);
        setServerKey(skR.data);
        try { const wR = await api.proofs.getWinner(id); setWinner(wR.data); } catch {}
        try { const lR = await api.proofs.listLosers(id); setLosers(lR.data.proofs); } catch {}
      } catch (err: any) { toast.error(err.message); }
    }
    load();
  }, [id]);

  async function runVerification() {
    if (!serverKey || !auction) {
      toast.error("Dati in caricamento...");
      return;
    }
    setLoading(true);
    try {
      const transcript = {
        auction_id: id,
        min_bid: auction.min_bid,
        max_bid: auction.max_bid,
        bid_step: auction.bid_step,
        bulletin_board: entries,
        winner: winner ? {
          bidder_id: winner.winner_id,
          bid_id: winner.bid_id,
          commitment_hex: bids.find(b => b.bid_id === winner.bid_id)?.commitment_hex ?? "",
          revealed_value: winner.revealed_value,
          proof_json: winner.proof_json,
        } : null,
        losers: losers.map(l => ({
          bidder_id: l.bidder_id,
          bid_id: l.bid_id,
          commitment_hex: bids.find(b => b.bid_id === l.bid_id)?.commitment_hex ?? "",
          proof_json: l.proof_json,
        })),
        // Passiamo l'oggetto esattamente come si aspetta la firma crittografica
        server_verifier: { verifying_key: serverKey.public_key_hex },
        pedersen_generators: { g: serverKey.pedersen_g_hex, h: serverKey.pedersen_h_hex },
      };

      const transcriptJson = JSON.stringify(transcript);
      console.log("📤 JSON inviato a Rust:", transcript);

      // Carichiamo il WASM scavalcando wasm.ts per vedere l'errore grezzo
      const mod = await import("@/wasm/auction_verifier.js");
      await mod.default();
      
      try {
        const r = mod.verify_transcript(transcriptJson);
        setResult(r as VerificationResult);
        toast.success(r.fully_valid ? "Asta completamente verificata!" : "La verifica ha riscontrato anomalie");
      } catch (rustError) {
        console.error("🚨 ERRORE DI VALIDAZIONE RUST:", rustError);
        toast.error("Il payload JSON è stato rifiutato da Rust. Controlla la console.");
      }

    } catch (err: any) {
      console.error("Errore generale:", err);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Verify Auction</h1>
        <p className="text-slate-400 mt-1 text-sm font-mono">{id}</p>
      </div>
      <VerificationPanel result={result} loading={loading} hasWasm={hasWasm} onVerify={runVerification} />
      <TranscriptViewer entries={entries} bids={bids} winner={winner} losers={losers} />
    </div>
  );
}
