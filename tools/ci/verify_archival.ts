import { execSync } from "child_process";
import { existsSync, readFileSync, readdirSync } from "fs";
import { join, relative, sep } from "path";

type RenameEntry = {
  type: "delete" | "rename";
  path: string;
  newPath?: string;
};

type LedgerEntry = {
  original_path?: string;
  replacement_path?: string | null;
  archive_path?: string;
  sha256?: string;
  hash?: string;
};

function run(command: string): string {
  return execSync(command, { stdio: ["ignore", "pipe", "pipe"] }).toString().trim();
}

function getRepoRoot(): string {
  return run("git rev-parse --show-toplevel");
}

function refExists(ref: string): boolean {
  try {
    run(`git rev-parse --verify ${ref}`);
    return true;
  } catch (error) {
    return false;
  }
}

function getBaseRef(): string {
  const envRef = process.env.BASE_REF?.trim();
  const fallbacks = ["origin/main", "main", "HEAD^", "HEAD~1"];

  if (envRef && envRef.length > 0) {
    if (refExists(envRef)) {
      return envRef;
    }
    console.warn(
      `BASE_REF=${envRef} is not available locally. Attempting standard fallbacks.`
    );
  }

  for (const candidate of fallbacks) {
    if (refExists(candidate)) {
      if (candidate === "origin/main") {
        return "origin/main";
      }
      console.warn(`Using fallback base reference: ${candidate}`);
      return candidate;
    }
  }

  throw new Error(
    "Unable to resolve a base reference for archival verification. Fetch your target branch or set BASE_REF explicitly."
  );
}

function parseDiff(baseRef: string): RenameEntry[] {
  let diffOutput = "";
  try {
    diffOutput = run(`git diff --name-status ${baseRef}...HEAD`);
  } catch (error) {
    console.error(`Failed to compute git diff against ${baseRef}. Ensure the ref exists and is fetched.`);
    throw error;
  }

  if (!diffOutput) {
    return [];
  }

  const entries: RenameEntry[] = [];
  const lines = diffOutput.split(/\r?\n/);
  for (const line of lines) {
    const parts = line.split(/\t+/);
    if (parts.length === 0) continue;
    const status = parts[0];
    if (status.startsWith("D")) {
      const path = parts[1];
      if (path) {
        entries.push({ type: "delete", path });
      }
    } else if (status.startsWith("R")) {
      const oldPath = parts[1];
      const newPath = parts[2];
      if (oldPath && newPath) {
        entries.push({ type: "rename", path: oldPath, newPath });
      }
    }
  }
  return entries;
}

function listDirectories(path: string, pattern: RegExp): string[] {
  if (!existsSync(path)) return [];
  return readdirSync(path, { withFileTypes: true })
    .filter((dirent) => dirent.isDirectory() && pattern.test(dirent.name))
    .map((dirent) => dirent.name);
}

function findArchiveForPath(repoRoot: string, targetPath: string): string | null {
  const archiveRoot = join(repoRoot, "archive");
  if (!existsSync(archiveRoot)) {
    return null;
  }

  const normalizedTarget = targetPath.split(/[\\/]/).join(sep) + ".tar.zst";
  const years = listDirectories(archiveRoot, /^\d{4}$/);
  for (const year of years) {
    const yearPath = join(archiveRoot, year);
    const months = listDirectories(yearPath, /^\d{2}$/);
    for (const month of months) {
      const candidate = join(yearPath, month, normalizedTarget);
      if (existsSync(candidate)) {
        return relative(repoRoot, candidate);
      }
    }
  }
  return null;
}

function loadLedger(repoRoot: string): LedgerEntry[] {
  const ledgerPath = join(repoRoot, "archive", "ledger.jsonl");
  if (!existsSync(ledgerPath)) {
    return [];
  }

  const contents = readFileSync(ledgerPath, "utf8");
  const entries: LedgerEntry[] = [];
  for (const line of contents.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    try {
      const parsed = JSON.parse(trimmed);
      entries.push(parsed as LedgerEntry);
    } catch (error) {
      console.warn(`Skipping malformed ledger line: ${trimmed}`);
    }
  }
  return entries;
}

function ensureLedgerEntry(
  ledgerEntries: LedgerEntry[],
  targetPath: string,
  replacementPath?: string
): LedgerEntry | null {
  for (const entry of ledgerEntries) {
    if (entry.original_path === targetPath) {
      if (replacementPath) {
        if (entry.replacement_path === replacementPath) {
          return entry;
        }
      } else if (entry.replacement_path === null || entry.replacement_path === undefined) {
        return entry;
      } else {
        // Accept ledger entries that explicitly state a replacement path even for deletions
        return entry;
      }
    }
  }
  return null;
}

function hasHash(entry: LedgerEntry): boolean {
  return Boolean(entry.sha256 || entry.hash);
}

function main() {
  const repoRoot = getRepoRoot();
  const baseRef = getBaseRef();
  const entries = parseDiff(baseRef);

  if (entries.length === 0) {
    console.log("No deletions or renames detected; archival verification skipped.");
    return;
  }

  const ledgerEntries = loadLedger(repoRoot);
  const errors: string[] = [];

  if (ledgerEntries.length === 0) {
    errors.push(
      "archive/ledger.jsonl is missing or empty. Provide ledger entries for archival operations."
    );
  }

  for (const entry of entries) {
    const archivePath = findArchiveForPath(repoRoot, entry.path);
    if (!archivePath) {
      errors.push(
        `Missing archive tarball for ${entry.path}. Expected archive/YYYY/MM/${entry.path}.tar.zst.`
      );
    }

    const ledgerEntry = ensureLedgerEntry(ledgerEntries, entry.path, entry.newPath);
    if (!ledgerEntry) {
      errors.push(
        entry.type === "rename"
          ? `No ledger entry found for rename ${entry.path} -> ${entry.newPath}.`
          : `No ledger entry found for deletion of ${entry.path}.`
      );
      continue;
    }

    if (!hasHash(ledgerEntry)) {
      errors.push(
        `Ledger entry for ${entry.path} is missing a hash (expected sha256 or hash field).`
      );
    }

    if (!ledgerEntry.archive_path && !archivePath) {
      errors.push(
        `Ledger entry for ${entry.path} does not record archive_path, and archive artifact is missing.`
      );
    }
  }

  if (errors.length > 0) {
    console.error("Archival verification failed:\n" + errors.map((e) => ` - ${e}`).join("\n"));
    console.error(
      "If this archival requirement must be skipped, obtain approval and apply the policy override label: ci-allow-archive-skip."
    );
    process.exit(1);
  }

  console.log("Archival verification passed.");
}

main();
