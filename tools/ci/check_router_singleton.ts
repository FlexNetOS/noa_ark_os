#!/usr/bin/env tsx
/**
 * CI guard that ensures no duplicate AI router entrypoints creep into the repo.
 * Only the canonical server router and the thin UI wrapper are allowed.
 */

import { execSync } from "node:child_process";

const allowedRouters = new Set([
  "server/ai/router.ts",
  "ui/vibe-kanban/server/ai/router.ts",
]);

function listRouters(): string[] {
  const output = execSync("git ls-files '**/router.ts'", { encoding: "utf8" });
  return output
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
    .filter((line) => !line.startsWith("archive/"));
}

function main() {
  const routers = listRouters();
  const unexpected = routers.filter((path) => !allowedRouters.has(path));

  if (unexpected.length > 0) {
    console.error("Found unsupported router entrypoints:");
    for (const path of unexpected) {
      console.error(`  - ${path}`);
    }
    console.error(
      "Only the canonical '@noa-ark/server/ai/router' and its UI wrapper are permitted. Remove or re-export from the shared gateway.",
    );
    process.exit(1);
  }
}

main();
