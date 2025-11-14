import { spawnSync } from "node:child_process";
import { ensureCommandAllowed } from "../../automation/command_policy.ts";

const MAX_GIT_BUFFER = 10 * 1024 * 1024;

export type GitOptions = {
  cwd?: string;
};

export function runGit(args: string[], options: GitOptions = {}) {
  ensureCommandAllowed(["git", ...args], { context: "offline-pr-queue" });
  const result = spawnSync("git", args, {
    stdio: ["ignore", "pipe", "pipe"],
    encoding: "utf8",
    maxBuffer: MAX_GIT_BUFFER,
    ...options,
  });

  if (result.error) {
    throw result.error;
  }

  if (result.status !== 0) {
    const error = new Error(`git ${args.join(" ")} failed: ${result.stderr?.trim() ?? "unknown"}`) as Error & {
      stdout?: string;
      stderr?: string;
      status?: number | null;
    };
    error.stdout = result.stdout;
    error.stderr = result.stderr;
    error.status = result.status;
    throw error;
  }

  return result.stdout.trim();
}

export function getCurrentBranch(): string {
  return runGit(["rev-parse", "--abbrev-ref", "HEAD"]);
}

export function checkoutBranch(branch: string): void {
  if (!branchExists(branch)) {
    throw new Error(`Cannot checkout missing branch: ${branch}`);
  }
  runGit(["checkout", branch]);
}

export function createBranch(branch: string, base: string): void {
  if (branchExists(branch)) {
    checkoutBranch(branch);
    return;
  }
  runGit(["checkout", "-b", branch, base]);
}

export function mergeBranch(branch: string): void {
  runGit(["merge", "--no-ff", branch]);
}

export function getDiff(base: string, head: string): string {
  return runGit(["diff", `${base}..${head}`]);
}

export function stage(paths: string[]): void {
  runGit(["add", ...paths]);
}

export function commit(message: string): void {
  runGit(["commit", "-m", message]);
}

export function branchExists(branch: string): boolean {
  try {
    runGit(["rev-parse", "--verify", branch]);
    return true;
  } catch (error) {
    return false;
  }
}
