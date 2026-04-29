import type { Config } from "tailwindcss";
export default {
  content: ["./src/**/*.{js,ts,jsx,tsx,mdx}"],
  theme: {
    extend: {
      colors: {
        brand: { DEFAULT: "#6366f1", dark: "#4f46e5", light: "#818cf8" },
        surface: { DEFAULT: "#0f172a", card: "#1e293b", border: "#334155" },
      },
    },
  },
  plugins: [],
} satisfies Config;