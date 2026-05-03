import { cp, copyFile, mkdir } from "node:fs/promises";
import path from "node:path";
import { defineConfig, type Plugin } from "vite";

function copyExtensionFiles(): Plugin {
  return {
    name: "copy-extension-files",
    async writeBundle() {
      const root = __dirname;
      const outDir = path.resolve(root, "out-extension");
      await mkdir(outDir, { recursive: true });
      await copyFile(
        path.resolve(root, "extension/manifest.json"),
        path.resolve(outDir, "manifest.json"),
      );
      await cp(
        path.resolve(root, "extension/assets"),
        path.resolve(outDir, "assets"),
        { recursive: true },
      );
    },
  };
}

export default defineConfig({
  root: path.resolve(__dirname, "extension"),
  base: "./",
  publicDir: false,
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  plugins: [copyExtensionFiles()],
  build: {
    outDir: "../out-extension",
    emptyOutDir: true,
    minify: false,
    sourcemap: false,
    rollupOptions: {
      input: {
        options: path.resolve(__dirname, "extension/options.html"),
        popup: path.resolve(__dirname, "extension/popup.html"),
      },
      output: {
        entryFileNames: "[name].js",
        chunkFileNames: "chunks/[name]-[hash].js",
        assetFileNames: "assets/[name]-[hash][extname]",
      },
    },
  },
});
