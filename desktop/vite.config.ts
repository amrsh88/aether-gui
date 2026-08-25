import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri serves the frontend on a fixed port and expects a static build in dist/.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      // src-tauri is watched by the Rust toolchain, not by Vite.
      ignored: ["**/src-tauri/**", "**/core/**"],
    },
  },
  build: {
    // WebView2 on Windows 10/11 is evergreen Chromium, so a modern target is safe.
    target: "chrome110",
    outDir: "dist",
    emptyOutDir: true,
    sourcemap: false,
  },
});
