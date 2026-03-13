import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import path from "path";
import { defineConfig } from "vite";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
    // visualizer({
    //   filename: './dist/stats.html',
    //   gzipSize: true,
    //   brotliSize: true,
    // }),
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
    rollupOptions: {
      external: ["mockServiceWorker.js"],
      output: {
        manualChunks(id) {
          if (id.includes("node_modules")) {
            const rules: Array<{
              match: string | string[];
              chunk: string;
              strict?: boolean;
            }> = [
              {
                match: [
                  "/react/",
                  "/react-dom/",
                  "react-aria",
                  "react-stately",
                  "heroui",
                ],
                chunk: "react",
              },
            ];

            // console.log(id);

            for (const { match, chunk } of rules) {
              if (Array.isArray(match)) {
                if (match.some((k) => id.includes(k))) return chunk;
              } else {
                if (id.includes(match)) return chunk;
              }
            }
          }
        },
      },
    },
  },
});
