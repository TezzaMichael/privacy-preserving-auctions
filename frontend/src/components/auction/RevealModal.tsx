"use client";
import { useState, useEffect, useRef } from "react";
import { toast } from "sonner";
import { api } from "@/lib/api";
import { useAuthStore } from "@/store/auth";
import { loadSecret } from "@/lib/crypto";
import { createProofOfOpeningWasm } from "@/lib/wasm";
import type { SealedBid, BidSecret } from "@/types";
import Modal from "@/components/ui/Modal";

interface Props { auctionId: string; myBid: SealedBid; onClose: () => void; onSuccess: () => void; }

export default function RevealModal({ auctionId, myBid, onClose, onSuccess }: Props) {
  const { token, secretKeyHex } = useAuthStore();
  const [secret, setSecret] = useState<BidSecret | null>(null);
  const [manualValue, setManualValue] = useState("");
  const [manualBlinding, setManualBlinding] = useState("");
  const [loading, setLoading] = useState(false);
  
  // Stati per gestire l'interfaccia dell'automazione
  const [autoSubmitting, setAutoSubmitting] = useState(false);
  const autoSubmitAttempted = useRef(false);

  const hasCryptoKey = !!secretKeyHex;

  useEffect(() => { 
    const loadedSecret = loadSecret(auctionId);
    setSecret(loadedSecret);

    // PILOTA AUTOMATICO: Se la chiave c'è e il segreto è in memoria, facciamo tutto da soli!
    if (loadedSecret && hasCryptoKey && !autoSubmitAttempted.current) {
      autoSubmitAttempted.current = true;
      setAutoSubmitting(true);
      
      // Aggiungiamo un ritardo di 1.5 secondi solo per la UX, 
      // così l'utente fa in tempo a leggere cosa sta succedendo sullo schermo
      setTimeout(() => {
        performReveal(loadedSecret.value, loadedSecret.blinding_hex);
      }, 1500);
    }
  }, [auctionId, hasCryptoKey]);

  // La logica core è stata estratta per poter essere chiamata sia in automatico che manualmente
  async function performReveal(value: number, blinding: string) {
    if (!token || !secretKeyHex) {
      toast.error("Sessione crittografica non valida in memoria. Effettua il logout e reinserisci la password.");
      return;
    }

    setLoading(true);
    try {
      const realProof = await createProofOfOpeningWasm(value, blinding, myBid.commitment_hex);
      if (!realProof) {
        throw new Error("Impossibile generare il proof crittografico di apertura tramite WASM.");
      }

      await api.proofs.revealWinner(token, auctionId, myBid.bid_id, value, realProof);
      toast.success("Rivendicazione inviata con successo!");
      onSuccess();
    } catch (err: any) {
      // Gestione degli errori specifica per il polling asincrono
      if (err.message.includes("Troppo presto") || err.message.includes("fuori sincrono")) {
        toast.error("Il server ha rifiutato la richiesta per mancata sincronizzazione temporale. Riprova.");
      } else if (err.message.includes("già") || err.message.includes("closed") || err.message.includes("submitted")) {
        toast.error("Impossibile rivelare: un altro utente ha già rivendicato la vittoria con un'offerta maggiore o uguale.");
      } else {
        toast.error(err.message);
      }
    } finally {
      setLoading(false);
      setAutoSubmitting(false);
    }
  }

  // Fallback per l'invio manuale nel caso in cui il sessionStorage sia vuoto
  async function handleManualReveal(e: React.FormEvent) {
    e.preventDefault();
    const v = secret ? secret.value : parseInt(manualValue);
    const b = secret ? secret.blinding_hex : manualBlinding;
    await performReveal(v, b);
  }

  return (
    <Modal title="Rivendicazione Offerta (Reveal)" onClose={onClose}>
      {!hasCryptoKey ? (
        <div className="bg-yellow-500/10 border-l-4 border-yellow-500 p-4 mb-4 text-sm text-yellow-300 rounded">
          <p className="font-bold text-yellow-400 mb-1">Sessione Crittografica Scaduta</p>
          <p className="mb-2">
            Hai ricaricato la pagina. Per motivi di sicurezza, la tua sessione crittografica temporanea è stata rimossa dalla memoria.
          </p>
          <p className="font-semibold">
            Chiudi questa finestra, esegui il Logout e accedi di nuovo per ripristinare le funzioni dell'asta.
          </p>
          <button type="button" className="btn-secondary w-full mt-4" onClick={onClose}>
            Chiudi
          </button>
        </div>
      ) : autoSubmitting ? (
        <div className="py-8 text-center flex flex-col items-center space-y-4">
          <div className="w-12 h-12 border-4 border-blue-500/30 border-t-blue-500 rounded-full animate-spin"></div>
          <div>
            <h3 className="text-lg font-medium text-slate-200">Elaborazione Automatica...</h3>
            <p className="text-sm text-slate-400 mt-2">
              Generazione della prova Zero-Knowledge in corso per la tua offerta da <strong>{secret?.value}</strong>.
              <br />Non chiudere la finestra.
            </p>
          </div>
        </div>
      ) : (
        <form onSubmit={handleManualReveal} className="space-y-4">
          {secret ? (
            <div className="bg-green-500/10 border border-green-500/20 rounded-lg p-3 text-sm text-green-300">
              Segreto recuperato dalla memoria sicura. Valore: <strong>{secret.value}</strong>
            </div>
          ) : (
            <>
              <div className="bg-orange-500/10 border border-orange-500/20 rounded-lg p-3 text-sm text-orange-300">
                Segreto non trovato in memoria. Inserisci manualmente i dati crittografici.
              </div>
              <div>
                <label className="block text-sm text-slate-400 mb-1">Bid Value</label>
                <input className="input" type="number" value={manualValue} onChange={e => setManualValue(e.target.value)} required={!secret} />
              </div>
              <div>
                <label className="block text-sm text-slate-400 mb-1">Blinding Hex</label>
                <input className="input mono" value={manualBlinding} onChange={e => setManualBlinding(e.target.value)} required={!secret} placeholder="64-char hex" />
              </div>
            </>
          )}
          <div className="flex gap-3">
            <button type="submit" className="btn-primary flex-1 justify-center" disabled={loading}>
              {loading ? "Invio in corso…" : "Invia Rivelazione"}
            </button>
            <button type="button" className="btn-secondary" onClick={onClose}>Annulla</button>
          </div>
        </form>
      )}
    </Modal>
  );
}