import { join } from "node:path";
import crypto from "node:crypto";
import { ensureDir, readJson, writeJson } from "./fs.ts";

const ROOT = "tools/offline_pr_queue";
const STATE_PATH = join(ROOT, "state.json");
const PR_DIR = join(ROOT, "prs");
const REVIEWS_DIR = join(ROOT, "reviews");
const CHECK_LOG_DIR = join(ROOT, "check_logs");
const TRIAGE_DIR = join(ROOT, "triage");
const SUGGESTIONS_DIR = join(ROOT, "suggestions");

export type QueuePaths = {
  root: string;
  state: string;
  prs: string;
  reviews: string;
  checks: string;
  triage: string;
  suggestions: string;
};

export function getPaths(): QueuePaths {
  ensureDir(PR_DIR);
  ensureDir(REVIEWS_DIR);
  ensureDir(CHECK_LOG_DIR);
  ensureDir(TRIAGE_DIR);
  ensureDir(SUGGESTIONS_DIR);
  return {
    root: ROOT,
    state: STATE_PATH,
    prs: PR_DIR,
    reviews: REVIEWS_DIR,
    checks: CHECK_LOG_DIR,
    triage: TRIAGE_DIR,
    suggestions: SUGGESTIONS_DIR,
  };
}

export function nextPrId(): number {
  const state = readJson<{ nextId: number }>(STATE_PATH, { nextId: 1 });
  const id = state.nextId;
  state.nextId += 1;
  writeJson(STATE_PATH, state);
  return id;
}

export function prFile(id: number): string {
  const { prs } = getPaths();
  return join(prs, `pr-${id}.json`);
}

export function reviewFile(id: number): string {
  const { reviews } = getPaths();
  return join(reviews, `pr-${id}.json`);
}

export function checkLogPath(id: number, name: string): string {
  const { checks } = getPaths();
  return join(checks, `pr-${id}-${name}.log`);
}

export function triagePath(id: number, name: string): string {
  const { triage } = getPaths();
  const unique = crypto.createHash("sha1").update(`${id}-${name}-${Date.now()}`).digest("hex").slice(0, 12);
  return join(triage, `pr-${id}-${name}-${unique}.json`);
}

export function suggestionPath(id: number): string {
  const { suggestions } = getPaths();
  return join(suggestions, `pr-${id}-ai-suggestion.md`);
}
