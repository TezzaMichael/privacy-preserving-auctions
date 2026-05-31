"use client";
import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { toast } from "sonner";
import { api } from "@/lib/api";
import { useAuthStore } from "@/store/auth";
import { deriveKeypairFromPassword } from "@/lib/crypto";

export default function LoginPage() {
  const router = useRouter();
  const setAuth = useAuthStore(s => s.setAuth);
  
  const [browserOwner, setBrowserOwner] = useState<string | null>(null);
  const [form, setForm] = useState({ username: "", password: "" });
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    // On load, check if this browser is already bound to a user
    const owner = localStorage.getItem("browser_owner");
    if (owner) {
      setBrowserOwner(owner);
      setForm(f => ({ ...f, username: owner })); // Lock the username
    }
  }, []);

  function handleFactoryReset() {
    const isConfirmed = window.confirm(
      "⚠️ CRITICAL WARNING ⚠️\n\nChanging users will PERMANENTLY delete all cryptographic keys and sealed bid secrets saved on this browser.\nIf you have active auctions, you will not be able to claim them or submit proofs.\n\nDo you want to proceed and reset the browser?"
    );

    if (isConfirmed) {
      localStorage.clear(); // Wipes the owner and all bid secrets
      setBrowserOwner(null);
      setForm({ username: "", password: "" });
      toast.success("Browser reset successfully. You can now register or log in as a new user.");
    }
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setLoading(true);
    try {
      // 1. Rebuild the private key in background AND check browser ownership lock
      const { secretKeyHex } = await deriveKeypairFromPassword(form.password, form.username);
      
      // 2. Perform backend login
      const r = await api.auth.login(form.username, form.password);
      const { jwt_token, user_id, username, public_key_hex } = r.data;
      
      // 3. Inject the rebuilt key into the store
      setAuth(jwt_token, { user_id, username, public_key_hex }, secretKeyHex);
      
      toast.success("Logged in successfully");
      router.push("/dashboard");
    } catch (err: any) {
      toast.error(err.message || "Sign in failed");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center p-4">
      <div className="card w-full max-w-md">
        <h1 className="text-2xl font-bold mb-2">Sign In</h1>
        <p className="text-slate-400 mb-6 text-sm">Privacy-preserving sealed-bid auction</p>
        
        {browserOwner ? (
          <div className="mb-6 bg-slate-800 p-4 rounded-xl border border-slate-700">
            <p className="text-sm text-slate-400">
              This browser is securely linked to:
            </p>
            <p className="text-xl font-bold text-brand-light mt-1 mb-4">
              {browserOwner}
            </p>
            
            <form onSubmit={handleSubmit} className="space-y-4">
              <div>
                <label className="block text-sm text-slate-400 mb-1">Password</label>
                <input 
                  className="input w-full" 
                  type="password" 
                  value={form.password} 
                  onChange={e => setForm(f => ({ ...f, password: e.target.value }))} 
                  required 
                  placeholder="Enter your password" 
                />
              </div>
              <button className="btn-primary w-full justify-center" type="submit" disabled={loading}>
                {loading ? "Signing in…" : "Sign In"}
              </button>
            </form>

            <div className="mt-6 pt-4 border-t border-slate-700 text-center">
              <button 
                type="button"
                onClick={handleFactoryReset}
                className="text-xs text-red-400 hover:text-red-300 underline transition-colors"
              >
                Not {browserOwner}? Click here to reset browser (Danger: Data Loss)
              </button>
            </div>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="bg-blue-500/10 border border-blue-500/20 rounded-lg p-3 mb-4 text-sm text-blue-300">
              This is a new browser session. The first user to log in will permanently link their cryptographic profile to this browser.
            </div>
            <div>
              <label className="block text-sm text-slate-400 mb-1">Username</label>
              <input 
                className="input w-full" 
                value={form.username} 
                onChange={e => setForm(f => ({ ...f, username: e.target.value }))} 
                required 
                placeholder="Enter your username"
              />
            </div>
            <div>
              <label className="block text-sm text-slate-400 mb-1">Password</label>
              <input 
                className="input w-full" 
                type="password" 
                value={form.password} 
                onChange={e => setForm(f => ({ ...f, password: e.target.value }))} 
                required 
                placeholder="Enter your password"
              />
            </div>
            
            <button className="btn-primary w-full justify-center" type="submit" disabled={loading}>
              {loading ? "Signing in…" : "Sign In & Lock Browser"}
            </button>

            <p className="mt-4 text-center text-sm text-slate-400">
              No account?{" "}
              <Link href="/register" className="text-brand-light hover:underline">Register</Link>
            </p>
          </form>
        )}
      </div>
    </div>
  );
}