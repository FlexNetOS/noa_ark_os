import { dirname } from "node:path";
import { writeFileSync, readdirSync, existsSync, readFileSync } from "node:fs";
import { readJson, writeJson, ensureDir } from "./fs.ts";
import {
  getPaths,
  nextPrId,
  prFile,
  reviewFile,
  checkLogPath,
  triagePath,
  suggestionPath,
} from "./state.ts";
import {
  getCurrentBranch,
  createBranch,
  checkoutBranch,
  mergeBranch,
  getDiff,
  stage,
  commit,
} from "./git.ts";
import { performChecks, CheckResult } from "./checks.ts";
import { analyzeLog } from "./analyzer.ts";

async function loadProvider() {
  const module = await import("../../server/ai/router.ts");
  return module.getProvider();
}

export type ReviewState = "approve" | "comment" | "request_changes";

export type OfflineReview = {
  reviewer: string;
  state: ReviewState;
  body: string;
  automated: boolean;
  submittedAt: string;
};

export type OfflinePr = {
  id: number;
  title: string;
  branch: string;
  base: string;
  createdAt: string;
  status: "open" | "merged";
  mergedAt?: string;
};

const REVIEW_STATES: ReviewState[] = ["approve", "comment", "request_changes"];

function slugify(text: string): string {
  return text.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 32);
}

export type CreatePrOptions = {
  title?: string;
  branch?: string;
  base?: string;
};

export function createPr({ title, branch, base }: CreatePrOptions): OfflinePr {
  getPaths();
  const id = nextPrId();
  const currentBase = base ?? getCurrentBranch();
  const safeTitle = slugify(title || `offline-pr-${id}`);
  const targetBranch = branch ?? `offline/pr/${id}-${safeTitle}`;
  createBranch(targetBranch, currentBase);

  const prData: OfflinePr = {
    id,
    title: title ?? `Offline PR ${id}`,
    branch: targetBranch,
    base: currentBase,
    createdAt: new Date().toISOString(),
    status: "open",
  };
  writeJson(prFile(id), prData);
  writeJson(reviewFile(id), []);
  return prData;
}

export type RecordReviewOptions = {
  id: number;
  reviewer: string;
  state: ReviewState;
  body: string;
  automated?: boolean;
};

export function recordReview({ id, reviewer, state, body, automated = false }: RecordReviewOptions): OfflineReview {
  if (!REVIEW_STATES.includes(state)) {
    throw new Error(`Invalid review state: ${state}`);
  }
  const reviews = readJson<OfflineReview[]>(reviewFile(id), []);
  const entry: OfflineReview = {
    reviewer,
    state,
    body,
    automated,
    submittedAt: new Date().toISOString(),
  };
  reviews.push(entry);
  writeJson(reviewFile(id), reviews);
  return entry;
}

type Summary = {
  passed: string[];
  failed: string[];
};

function summarizeChecks(results: Array<Pick<CheckResult, "name" | "status">>): Summary {
  const summary: Summary = { passed: [], failed: [] };
  for (const result of results) {
    if (result.status === "passed") {
      summary.passed.push(result.name);
    } else {
      summary.failed.push(result.name);
    }
  }
  return summary;
}

function aiReviewPrompt(pr: OfflinePr, diff: string): string {
  return `Review the following diff for PR ${pr.id} (${pr.title}). Focus on safety, offline readiness, and missing tests. Respond with a short paragraph and end with APPROVE or REJECT.\n\n${diff}`;
}

export async function runAiReview(id: number): Promise<OfflineReview> {
  const pr = readJson<OfflinePr>(prFile(id));
  const diff = getDiff(pr.base, pr.branch);
  const provider = await loadProvider();
  if (!provider || !provider.isConfigured()) {
    const body = diff.length === 0 ? "No diff to review." : "AI provider unavailable; manual review required.";
    return recordReview({ id, reviewer: "ai-offline", state: "comment", body, automated: true });
  }

  const response = await provider.completePrompt(aiReviewPrompt(pr, diff));
  const state: ReviewState = /approve/i.test(response) && !/reject/i.test(response) ? "approve" : "comment";
  return recordReview({ id, reviewer: `ai-${provider.name}`, state, body: response.trim(), automated: true });
}

export type TriageTicket = {
  pr: number;
  check: string;
  status: CheckResult["status"];
  analysis: ReturnType<typeof analyzeLog>;
  createdAt: string;
  logPath: string;
  mode: "online" | "offline";
  note: string;
  path: string;
};

function createTriageTicket(id: number, result: CheckResult, logPath: string): TriageTicket {
  const logData = readJson<unknown>(logPath, null);
  const content = logData !== null ? JSON.stringify(logData, null, 2) : "";
  const analysis = analyzeLog(result.name, content);
  const path = triagePath(id, result.name);
  const online = (process.env.ONLINE_GITHUB_MODE ?? "false").toLowerCase() === "true";
  const ticket: TriageTicket = {
    pr: id,
    check: result.name,
    status: result.status,
    analysis,
    createdAt: new Date().toISOString(),
    logPath,
    mode: online ? "online" : "offline",
    note: online
      ? "Online triage would open a GitHub issue; offline queue captured the analysis instead."
      : "Offline triage stored locally.",
    path,
  };
  writeJson(path, ticket);
  return ticket;
}

export type MergeResult = {
  merged: boolean;
  summary: Summary;
  aiReview: OfflineReview;
};

export async function runChecksAndMaybeMerge(id: number, { autoApplyAi = true } = {}): Promise<MergeResult> {
  const pr = readJson<OfflinePr>(prFile(id));

  if (pr.status === "merged") {
    const reviews = loadReviews(id);
    const automated = [...reviews].reverse().find((review) => review.automated);
    return {
      merged: true,
      summary: { passed: [], failed: [] },
      aiReview:
        automated ?? {
          reviewer: "ai-offline",
          state: "approve",
          body: "PR already merged; skipped duplicate run.",
          automated: true,
          submittedAt: new Date().toISOString(),
        },
    };
  }

  checkoutBranch(pr.branch);
  const results = performChecks(id, checkLogPath);
  const summary = summarizeChecks(results);

  for (const result of results) {
    if (result.status !== "passed") {
      createTriageTicket(id, result, checkLogPath(id, result.name));
    }
  }

  const aiReview = await runAiReview(id);
  const allGreen = summary.failed.length === 0 && aiReview.state !== "request_changes";

  if (!allGreen) {
    return { merged: false, summary, aiReview };
  }

  await enforceConventionalCommit();

  if (autoApplyAi) {
    await applyAiSuggestions(id, pr);
  }

  checkoutBranch(pr.base);
  mergeBranch(pr.branch);
  const updated: OfflinePr = { ...pr, status: "merged", mergedAt: new Date().toISOString() };
  writeJson(prFile(id), updated);
  return { merged: true, summary, aiReview };
}

async function applyAiSuggestions(id: number, pr: OfflinePr) {
  const provider = await loadProvider();
  if (!provider || !provider.isConfigured()) {
    return;
  }
  const diff = getDiff(pr.base, pr.branch);
  if (!diff.trim()) {
    return;
  }
  const prompt = `Suggest succinct improvements for the applied diff. If no changes are required, reply with NONE. Otherwise, propose incremental improvements in markdown bullet points.`;
  const suggestions = await provider.completePrompt(`${prompt}\n\n${diff}`);
  if (/^none$/i.test(suggestions.trim())) {
    return;
  }
  const path = suggestionPath(id);
  ensureDir(dirname(path));
  if (existsSync(path)) {
    const existing = readFileSync(path, "utf8");
    if (existing.includes(suggestions.trim())) {
      return;
    }
  }
  const header = `# AI Suggestions for PR ${id}\n\nGenerated: ${new Date().toISOString()}\n\n${suggestions.trim()}\n`;
  writeFileSync(path, header);
  stage([path]);
  commit(`ci(auto-apply): record ai suggestions for pr ${id}`);
}

export function loadReviews(id: number): OfflineReview[] {
  return readJson<OfflineReview[]>(reviewFile(id), []);
}

export function loadPr(id: number): OfflinePr {
  return readJson<OfflinePr>(prFile(id));
}

export function listTriageTickets(): TriageTicket[] {
  const { triage } = getPaths();
  const entries = readdirSync(triage).filter((file) => file.endsWith(".json"));
  return entries
    .map((file) => {
      try {
        return readJson<TriageTicket | null>(`${triage}/${file}`, null);
      } catch (error) {
        return null;
      }
    })
    .filter((value): value is TriageTicket => Boolean(value));
}

async function enforceConventionalCommit() {
  const module = await import("../../commit_copilot/conventional.ts");
  module.enforceLatestCommit();
}
