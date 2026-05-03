import path from "node:path";
import { defineConfig } from "vite";

export default defineConfig({
  root: path.resolve(__dirname, "extension"),
  base: "./",
  publicDir: false,
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  build: {
    outDir: "../out-extension",
    emptyOutDir: false,
    minify: false,
    sourcemap: false,
    lib: {
      entry: path.resolve(__dirname, "extension/src/content.ts"),
      formats: ["iife"],
      name: "DictDeckContentScript",
      fileName: () => "content.js",
    },
  },
});
