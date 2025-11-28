import test from "node:test";
import assert from "node:assert";
import { listTriageTickets, TriageTicket } from "../lib/queue.ts";
import { triagePath, getPaths } from "../lib/state.ts";
import { writeJson } from "../lib/fs.ts";
import { writeFileSync, rmSync } from "node:fs";
import { join } from "node:path";

function cleanup(path: string): void {
  try {
    rmSync(path, { force: true });
  } catch (error) {
    // ignore cleanup failures for test artifacts
  }
}

test("listTriageTickets returns only parsed tickets", () => {
  const { triage } = getPaths();
  const stray = join(triage, "notes.txt");
  writeFileSync(stray, "not json");

  const ticketPath = triagePath(999, "lint");
  const ticket: TriageTicket = {
    pr: 999,
    check: "lint",
    status: "failed",
    analysis: {
      check: "lint",
      summary: "lint failed",
      suggestions: ["fix lint"],
      matched: null,
    },
    createdAt: new Date().toISOString(),
    logPath: "tools/offline_pr_queue/check_logs/pr-999-lint.log",
    mode: "offline",
    note: "Offline triage stored locally.",
    path: ticketPath,
  };

  writeJson(ticketPath, ticket);

  try {
    const tickets = listTriageTickets();
    const found = tickets.find((entry) => entry.pr === 999);
    assert.ok(found, "expected to find inserted ticket");
    assert.strictEqual(found?.path, ticketPath);
    assert.strictEqual(tickets.every((entry) => entry.path.endsWith(".json")), true);
  } finally {
    cleanup(ticketPath);
    cleanup(stray);
  }
});
