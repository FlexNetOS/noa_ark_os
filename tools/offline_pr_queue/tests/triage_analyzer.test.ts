import test from "node:test";
import assert from "node:assert";
import { analyzeLog } from "../lib/analyzer.ts";

test("analyzeLog detects syntax errors", () => {
  const result = analyzeLog("lint", "SyntaxError: Unexpected token");
  assert.strictEqual(result.check, "lint");
  assert.ok(result.summary.includes("Syntax"));
  assert.ok(result.suggestions.length > 0);
});

test("analyzeLog falls back when no heuristic matches", () => {
  const result = analyzeLog("test", "All good");
  assert.strictEqual(result.matched, null);
  assert.ok(result.summary.includes("Investigation"));
});
