import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import path from "path";
import { defineConfig } from "vite";
import { visualizer } from "rollup-plugin-visualizer";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
    visualizer({
      filename: "./dist/stats.html",
      gzipSize: true,
      brotliSize: true,
    }),
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    allowedHosts: true,
  },
  build: {
    outDir: "out",
    chunkSizeWarningLimit: 1000,
    rollupOptions: {
      external: ["mockServiceWorker.js"],
      output: {
        manualChunks(id) {
          if (id.includes("node_modules")) {
            if (
              id.includes("react") ||
              id.includes("react-dom") ||
              id.includes("react-aria") ||
              id.includes("react-stately")
            ) {
              return "vendor-react";
            }
            if (id.includes("@radix-ui")) {
              return "vendor-radix";
            }
            if (id.includes("framer-motion")) {
              return "vendor-motion";
            }
            if (id.includes("lucide-react")) {
              return "vendor-lucide";
            }
            if (
              id.includes("streamdown") ||
              id.includes("rehype-raw") ||
              id.includes("parse5") ||
              id.includes("micromark")
            ) {
              return "vendor-markdown";
            }
            return "vendor-others";
          }
        },
      },
    },
  },
});
