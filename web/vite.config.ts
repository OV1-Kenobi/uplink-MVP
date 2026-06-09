import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import { VitePWA } from "vite-plugin-pwa";

export default defineConfig({
  plugins: [
    // WASM support — required for uplink-core wasm-bindgen bundle
    wasm(),

    react(),

    VitePWA({
      registerType: "autoUpdate",
      includeAssets: ["favicon.ico", "apple-touch-icon.png"],
      manifest: {
        name: "Uplink",
        short_name: "Uplink",
        description:
          "Nostr-native streaming-sats coordination with Stable-Channel wallets",
        theme_color: "#0f0f23",
        background_color: "#0f0f23",
        display: "standalone",
        orientation: "portrait",
        icons: [
          { src: "pwa-192x192.png", sizes: "192x192", type: "image/png" },
          { src: "pwa-512x512.png", sizes: "512x512", type: "image/png" },
        ],
      },
      workbox: {
        // Cache the wasm bundle for offline use
        globPatterns: ["**/*.{js,css,html,wasm}"],
        // The uplink-core wasm bundle is ~3 MB, above workbox's 2 MiB default;
        // raise the precache ceiling so it is available offline.
        maximumFileSizeToCacheInBytes: 5 * 1024 * 1024,
        runtimeCaching: [
          {
            urlPattern: /^https:\/\/relay\./,
            handler: "NetworkOnly",
            // WebSocket relay connections are never cached
          },
        ],
      },
    }),
  ],

  // Required for top-level await in wasm-bindgen generated glue
  build: {
    target: "esnext",
  },

  server: {
    headers: {
      // Required for SharedArrayBuffer (future: wasm-threads)
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
    },
  },
});
