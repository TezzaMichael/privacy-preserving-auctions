"use client";
import { useState, useEffect } from "react";
import { toast } from "sonner";
import { api } from "@/lib/api";
import { useAuthStore } from "@/store/auth";
import { loadSecret } from "@/lib/crypto";
import { createProofOfOpeningWasm } from "@/lib/wasm";
import type { SealedBid, BidSecret } from "@/types";
import Modal from "@/components/ui/Modal";

interface Props { auctionId: string; myBid: SealedBid; onClose: () => void; onSuccess: () => void; }

export default function RevealModal({ auctionId, myBid, onClose, onSuccess }: Props) {
  // Aggiunto secretKeyHex per verificare lo stato della sessione crittografica
  const { token, secretKeyHex } = useAuthStore();
  const [secret, setSecret] = useState<BidSecret | null>(null);
  const [manualValue, setManualValue] = useState("");
  const [manualBlinding, setManualBlinding] = useState("");
  const [loading, setLoading] = useState(false);

  // Controllo presenza chiave
  const hasCryptoKey = !!secretKeyHex;

  useEffect(() => { setSecret(loadSecret(auctionId)); }, [auctionId]);

  async function handleReveal(e: React.FormEvent) {
    e.preventDefault();
    
    if (!token || !secretKeyHex) {
      toast.error("Sessione crittografica non valida in memoria. Effettua il logout e reinserisci la password.");
      return;
    }
    
    setLoading(true);
    try {
      const value = secret ? secret.value : parseInt(manualValue);
      const blinding = secret ? secret.blinding_hex : manualBlinding;
      
      // Generazione del proof crittografico tramite WASM
      const realProof = await createProofOfOpeningWasm(value, blinding, myBid.commitment_hex);
      if (!realProof) {
        throw new Error("Impossibile generare il proof crittografico di apertura tramite WASM.");
      }

      await api.proofs.revealWinner(token, auctionId, myBid.bid_id, value, realProof);
      toast.success("Winner reveal submitted");
      onSuccess();
    } catch (err: any) {
      // Se l'errore riguarda un'asta già chiusa o un vincitore già dichiarato:
      if (err.message.includes("già") || err.message.includes("closed") || err.message.includes("submitted")) {
        toast.error("Impossibile rivelare: un altro utente ha già rivendicato la vittoria prima di te (Tie-break perso).");
      } else {
        toast.error(err.message);
      }
    } finally {
      setLoading(false);
    }
  }

  return (
    <Modal title="Reveal Winner Bid" onClose={onClose}>
      {!hasCryptoKey ? (
        <div className="bg-yellow-500/10 border-l-4 border-yellow-500 p-4 mb-4 text-sm text-yellow-300 rounded">
          <p className="font-bold text-yellow-400 mb-1">Sessione Crittografica Scaduta</p>
          <p className="mb-2">
            Hai ricaricato la pagina. Per motivi di sicurezza, la tua sessione crittografica temporanea è stata rimossa dalla memoria del browser.
          </p>
          <p className="font-semibold">
            Per favore, chiudi questa finestra, esegui il Logout e accedi di nuovo per ripristinare le funzioni dell'asta.
          </p>
          <button type="button" className="btn-secondary w-full mt-4" onClick={onClose}>
            Chiudi
          </button>
        </div>
      ) : (
        <form onSubmit={handleReveal} className="space-y-4">
          {secret ? (
            <div className="bg-green-500/10 border border-green-500/20 rounded-lg p-3 text-sm text-green-300">
              Secret found in session storage. Value: <strong>{secret.value}</strong>
            </div>
          ) : (
            <>
              <div className="bg-orange-500/10 border border-orange-500/20 rounded-lg p-3 text-sm text-orange-300">
                Secret not found in storage. Enter manually.
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
              {loading ? "Submitting…" : "Submit Reveal"}
            </button>
            <button type="button" className="btn-secondary" onClick={onClose}>Cancel</button>
          </div>
        </form>
      )}
    </Modal>
  );
}