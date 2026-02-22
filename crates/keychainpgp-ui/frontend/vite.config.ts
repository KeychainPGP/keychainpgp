import { defineConfig } from "vite";
import { paraglideVitePlugin } from "@inlang/paraglide-js";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import path from "path";
import { readFileSync } from "fs";

const pkg = JSON.parse(readFileSync("./package.json", "utf-8"));

export default defineConfig({
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },

  plugins: [
    paraglideVitePlugin({
      project: "./project.inlang",
      outdir: "./src/lib/paraglide",
    }),
    svelte(),
    tailwindcss(),
  ],

  resolve: {
    alias: {
      $lib: path.resolve("./src/lib"),
    },
  },

  // Prevent vite from obscuring Rust errors
  clearScreen: false,

  // Tauri expects a fixed port; fail if that port is not available
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },

  build: {
    outDir: "build",
    // Tauri uses Chromium on Windows and WebKit on macOS/Linux
    target: process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari13",
    // Don't minify for debug builds
    minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
    // Produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
