import test from "node:test";
import assert from "node:assert";
import { writeJson, readJson } from "../lib/fs.ts";
import { prFile, reviewFile } from "../lib/state.ts";
import { recordReview, loadReviews, OfflineReview } from "../lib/queue.ts";
import { rmSync } from "node:fs";

function freshPrId(): number {
  return Number(process.hrtime.bigint() % BigInt(100000));
}

test("recordReview appends to offline ledger", () => {
  const id = freshPrId();
  const prPath = prFile(id);
  const reviewPath = reviewFile(id);
  writeJson(prPath, {
    id,
    title: "Test PR",
    branch: `offline/pr/${id}`,
    base: "main",
    createdAt: new Date().toISOString(),
    status: "open",
  });
  writeJson(reviewPath, []);

  try {
    const entry = recordReview({ id, reviewer: "tester", state: "comment", body: "looks good" });
    assert.strictEqual(entry.reviewer, "tester");

    const reviews = loadReviews(id);
    assert.strictEqual(reviews.length, 1);
    assert.strictEqual(reviews[0].body, "looks good");

    const raw = readJson<OfflineReview[]>(reviewPath, []);
    assert.strictEqual(raw.length, 1);
  } finally {
    rmIfExists(reviewPath);
    rmIfExists(prPath);
  }
});

function rmIfExists(path: string): void {
  try {
    rmSync(path, { force: true });
  } catch (error) {
    // ignore cleanup errors
  }
}
