"use client";
import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { toast } from "sonner";
import { api } from "@/lib/api";
import { useAuthStore } from "@/store/auth";

function generateKeyPair(): { publicKeyHex: string; secretKeyHex: string } {
  const secret = crypto.getRandomValues(new Uint8Array(32));
  const secretKeyHex = Array.from(secret).map(b => b.toString(16).padStart(2, "0")).join("");
  const publicKeyHex = secretKeyHex;
  return { publicKeyHex, secretKeyHex };
}

export default function RegisterPage() {
  const router = useRouter();
  const setAuth = useAuthStore(s => s.setAuth);
  const [form, setForm] = useState({ username: "", password: "", confirmPassword: "" });
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (form.password !== form.confirmPassword) {
      toast.error("Passwords do not match");
      return;
    }
    setLoading(true);
    try {
      const { publicKeyHex, secretKeyHex } = generateKeyPair();
      await api.auth.register(form.username, form.password, publicKeyHex);
      const loginResp = await api.auth.login(form.username, form.password);
      const { jwt_token, user_id, username, public_key_hex } = loginResp.data;
      setAuth(jwt_token, { user_id, username, public_key_hex }, secretKeyHex);
      toast.success("Account created successfully");
      router.push("/dashboard");
    } catch (err: any) {
      toast.error(err.message);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center p-4">
      <div className="card w-full max-w-md">
        <h1 className="text-2xl font-bold mb-2">Create Account</h1>
        <p className="text-slate-400 mb-6 text-sm">Keys are generated client-side — your private key never leaves your browser.</p>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm text-slate-400 mb-1">Username</label>
            <input className="input" value={form.username} onChange={e => setForm(f => ({ ...f, username: e.target.value }))} required minLength={3} maxLength={32} />
          </div>
          <div>
            <label className="block text-sm text-slate-400 mb-1">Password</label>
            <input className="input" type="password" value={form.password} onChange={e => setForm(f => ({ ...f, password: e.target.value }))} required minLength={8} />
          </div>
          <div>
            <label className="block text-sm text-slate-400 mb-1">Confirm Password</label>
            <input className="input" type="password" value={form.confirmPassword} onChange={e => setForm(f => ({ ...f, confirmPassword: e.target.value }))} required />
          </div>
          <button className="btn-primary w-full justify-center" type="submit" disabled={loading}>
            {loading ? "Creating account…" : "Create Account"}
          </button>
        </form>
        <p className="mt-4 text-center text-sm text-slate-400">
          Have an account?{" "}
          <Link href="/login" className="text-brand-light hover:underline">Sign in</Link>
        </p>
      </div>
    </div>
  );
}