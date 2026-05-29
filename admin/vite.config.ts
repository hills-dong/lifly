import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Standalone admin SPA. In dev it proxies /api to the backend; point
// VITE_API_TARGET at wherever the Lifly server runs (default: local dev server).
export default defineConfig({
  // Base public path. Dev serves at "/"; the Docker build sets VITE_BASE=/admin/
  // so the backend can host the SPA under the /admin path prefix.
  base: process.env.VITE_BASE || "/",
  plugins: [react()],
  server: {
    port: 5273,
    proxy: {
      "/api": {
        target: process.env.VITE_API_TARGET || "http://localhost:8090",
        changeOrigin: true,
      },
    },
  },
});
