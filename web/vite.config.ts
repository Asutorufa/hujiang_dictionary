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
        // Only pin the long-lived framework code into stable vendor chunks.
        // Everything else (Markdown/KaTeX/Shiki, motion, …) is left to the
        // bundler so it is split along the lazy route/component boundaries
        // instead of being modulepreloaded on the first visit.
        manualChunks(id) {
          if (
            /[\\/]node_modules[\\/](react|react-dom|scheduler)[\\/]/.test(id)
          ) {
            return "vendor-react";
          }
          if (id.includes("/node_modules/@radix-ui/")) {
            return "vendor-radix";
          }
        },
      },
    },
  },
});
