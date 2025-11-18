import { runGit } from "../offline_pr_queue/lib/git.ts";

const CONVENTIONAL_PATTERN = /^(build|ci|docs|feat|fix|perf|refactor|style|test|chore|revert)(\([^)]+\))?!?: .+/;

function shouldEnforce(message: string): boolean {
  if (!message) {
    return false;
  }
  const normalized = message.trim();
  if (normalized.length === 0) {
    return false;
  }
  if (normalized.startsWith("Merge ")) {
    return false;
  }
  if (normalized.startsWith("Revert ")) {
    return false;
  }
  if (/^dependabot/i.test(normalized)) {
    return false;
  }
  return true;
}

async function loadProvider() {
  const module = await import("../../server/ai/router.ts");
  return module.getProvider();
}

export function enforceLatestCommit(): void {
  const message = runGit(["log", "-1", "--pretty=%B"]).split("\n")[0]?.trim() ?? "";
  if (!shouldEnforce(message)) {
    return;
  }
  if (!CONVENTIONAL_PATTERN.test(message)) {
    throw new Error(`Latest commit message is not Conventional Commits compliant: "${message}"`);
  }
}

export function isConventionalCommit(message: string): boolean {
  return CONVENTIONAL_PATTERN.test(message.trim());
}

export async function suggestCommitMessage(): Promise<string> {
  const diff = runGit(["diff", "--staged"]);
  if (!diff.trim()) {
    return "chore: update workspace metadata";
  }
  const provider = await loadProvider();
  if (!provider || !provider.isConfigured()) {
    return fallbackSuggestion(diff);
  }
  const prompt = `Generate a Conventional Commit message for the following staged diff. Respond with a single line Conventional Commit formatted summary. If unsure, default to chore(scope): summary.\n\n${diff}`;
  const suggestion = await provider.completePrompt(prompt);
  const firstLine = suggestion.split("\n")[0].trim();
  return isConventionalCommit(firstLine) ? firstLine : fallbackSuggestion(diff);
}

function fallbackSuggestion(diff: string): string {
  const scope = deriveScope(diff);
  return `chore(${scope}): update staged changes`;
}

function deriveScope(diff: string): string {
  const match = diff.match(/^diff --git a\/([^/\n]+)\//m);
  if (match) {
    return match[1].replace(/[^a-z0-9-]/gi, "-").toLowerCase();
  }
  return "repo";
}
