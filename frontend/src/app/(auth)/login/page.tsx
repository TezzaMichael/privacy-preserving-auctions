"use client";
import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { toast } from "sonner";
import { api } from "@/lib/api";
import { useAuthStore } from "@/store/auth";
import { deriveKeypairFromPassword } from "@/lib/crypto";

export default function LoginPage() {
  const router = useRouter();
  const { setAuth, clearAuth, user } = useAuthStore();
  
  const [form, setForm] = useState({ username: "", password: "" });
  const [loading, setLoading] = useState(false);

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

  function handleLogout() {
    clearAuth();
    sessionStorage.clear(); // Wipes local tab memory safely
    toast.success("Logged out successfully");
  }

  return (
    <div className="min-h-screen flex items-center justify-center p-4">
      <div className="card w-full max-w-md">
        <h1 className="text-2xl font-bold mb-2">Sign In</h1>
        <p className="text-slate-400 mb-6 text-sm">Privacy-preserving sealed-bid auction</p>
        
        {}
        {user ? (
          <div className="mb-6 bg-slate-800 p-4 rounded-xl border border-slate-700">
            <p className="text-sm text-slate-400">You are currently logged in as:</p>
            <p className="text-xl font-bold text-brand-light mt-1 mb-4">{user.username}</p>
            
            <button 
              className="btn-primary w-full justify-center mb-3" 
              onClick={() => router.push("/dashboard")}
            >
              Go to Dashboard
            </button>
            <button 
              className="btn-secondary w-full justify-center" 
              onClick={handleLogout}
            >
              Sign out to switch accounts
            </button>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="space-y-4">
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
              {loading ? "Signing in…" : "Sign In"}
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
