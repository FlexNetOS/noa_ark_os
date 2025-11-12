/**
 * Vitest configuration for the Vibe Kanban app, covering UI and server utilities.
 */

import { defineConfig } from "vitest/config";
import { resolve } from "node:path";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    setupFiles: ["./vitest.setup.ts"],
    globals: true,
  },
  server: {
    fs: {
      allow: [resolve(__dirname, "./"), resolve(__dirname, "../..")],
    },
  },
  resolve: {
    alias: {
      "@/": `${resolve(__dirname, "./")}/`,
      "server/ai": resolve(__dirname, "../..", "server/ai"),
      zod: resolve(__dirname, "node_modules/zod"),
    },
  },
});
