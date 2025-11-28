import test from "node:test";
import assert from "node:assert";
import { runChecksAndMaybeMerge, OfflineReview } from "../lib/queue.ts";
import { prFile, reviewFile } from "../lib/state.ts";
import { writeJson } from "../lib/fs.ts";
import { rmSync } from "node:fs";

test("runChecksAndMaybeMerge short-circuits merged PRs", async () => {
  const id = Number(process.hrtime.bigint() % BigInt(100000));
  const prPath = prFile(id);
  const reviewPath = reviewFile(id);
  const review: OfflineReview = {
    reviewer: "ai-llm",
    state: "approve",
    body: "looks good",
    automated: true,
    submittedAt: new Date().toISOString(),
  };
  writeJson(prPath, {
    id,
    title: "Merged PR",
    branch: "offline/pr/test",
    base: "main",
    createdAt: new Date().toISOString(),
    status: "merged",
    mergedAt: new Date().toISOString(),
  });
  writeJson(reviewPath, [review]);

  try {
    const result = await runChecksAndMaybeMerge(id);
    assert.strictEqual(result.merged, true);
    assert.deepStrictEqual(result.summary, { passed: [], failed: [] });
    assert.strictEqual(result.aiReview.reviewer, review.reviewer);
  } finally {
    rmSync(prPath, { force: true });
    rmSync(reviewPath, { force: true });
  }
});
