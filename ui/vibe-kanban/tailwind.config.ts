import type { Config } from "tailwindcss";

const config: Config = {
  content: [
    "./app/**/*.{js,ts,jsx,tsx,mdx}",
    "./components/**/*.{js,ts,jsx,tsx,mdx}",
    "./ui/**/*.{js,ts,jsx,tsx,mdx}"
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: ["Inter", "var(--font-inter)", "ui-sans-serif", "system-ui"],
      },
      colors: {
        background: "#050609",
        surface: "#0f1015",
        panel: "#13141d",
        accent: {
          600: "#4f46e5",
          500: "#6366f1",
          400: "#818cf8",
          300: "#a5b4fc",
        },
      },
      boxShadow: {
        glow: "0 18px 45px rgba(99, 102, 241, 0.45)",
        card: "0 24px 60px -35px rgba(56, 189, 248, 0.45)",
      },
      backgroundSize: {
        "200%": "200% 200%",
      },
      keyframes: {
        shimmer: {
          "0%": { backgroundPosition: "0% 50%" },
          "100%": { backgroundPosition: "100% 50%" },
        },
      },
      animation: {
        shimmer: "shimmer 12s ease-in-out infinite",
      },
    },
  },
  plugins: [],
};

export default config;
