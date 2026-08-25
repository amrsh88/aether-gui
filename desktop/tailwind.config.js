/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        base: "#0A0A0F",
        surface: "#13131A",
        elevated: "#1A1A23",
        hairline: "rgba(255,255,255,0.06)",
        ink: {
          DEFAULT: "#F4F4F5",
          dim: "#A1A1AA",
          faint: "#71717A",
        },
        idle: {
          from: "#6366F1",
          to: "#8B5CF6",
        },
        live: {
          from: "#06B6D4",
          to: "#10B981",
        },
        danger: "#EF4444",
        warn: "#F59E0B",
      },
      fontFamily: {
        sans: ["Inter", "Segoe UI", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "Cascadia Code", "Consolas", "monospace"],
      },
      boxShadow: {
        glow: "0 0 40px -8px var(--glow-color, rgba(99,102,241,0.55))",
        lift: "0 12px 32px -12px rgba(0,0,0,0.7)",
      },
      borderRadius: {
        card: "16px",
      },
    },
  },
  plugins: [],
};
