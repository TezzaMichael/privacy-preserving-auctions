"use client";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { LogOut, Shield } from "lucide-react";
import { useAuthStore } from "@/store/auth";

export default function Navbar() {
  const router = useRouter();
  const { user, clearAuth } = useAuthStore();

  function logout() {
    clearAuth();
    router.push("/login");
  }

  return (
    <nav className="border-b border-surface-border bg-surface-card/50 backdrop-blur sticky top-0 z-40">
      <div className="container mx-auto px-4 max-w-7xl flex items-center justify-between h-16">
        <Link href="/dashboard" className="flex items-center gap-2 font-bold text-lg">
          <Shield size={20} className="text-brand" />
          <span>ZK Auction</span>
        </Link>
        <div className="flex items-center gap-4">
          <span className="text-slate-400 text-sm">{user?.username}</span>
          <button onClick={logout} className="btn-ghost">
            <LogOut size={16} /> Sign out
          </button>
        </div>
      </div>
    </nav>
  );
}