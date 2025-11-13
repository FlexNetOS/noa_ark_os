import { createHash } from "crypto";
import { readFileSync, readdirSync, statSync } from "fs";
import { join, relative } from "path";
import { execSync } from "child_process";

type Config = {
  ignore: string[];
  allowedDuplicateGroups: string[][];
};

function getRepoRoot(): string {
  return execSync("git rev-parse --show-toplevel", {
    stdio: ["ignore", "pipe", "pipe"],
  })
    .toString()
    .trim();
}

function loadConfig(repoRoot: string): Config {
  const configPath = join(repoRoot, "tools", "ci", "check_duplicate_content.config.json");
  try {
    const raw = readFileSync(configPath, "utf8");
    const parsed = JSON.parse(raw) as Config;
    parsed.ignore = parsed.ignore || [];
    parsed.allowedDuplicateGroups = parsed.allowedDuplicateGroups || [];
    return parsed;
  } catch (error) {
    throw new Error(`Unable to read duplicate content config at ${configPath}: ${error}`);
  }
}

function normalizePath(path: string): string {
  return path.split("\\").join("/");
}

function shouldIgnore(relPath: string, ignoreList: string[]): boolean {
  for (const ignore of ignoreList) {
    const normalizedIgnore = normalizePath(ignore).replace(/\/$/, "");
    if (normalizedIgnore.length === 0) continue;
    if (
      relPath === normalizedIgnore ||
      relPath.startsWith(`${normalizedIgnore}/`)
    ) {
      return true;
    }

    const ignoreSegments = normalizedIgnore.split("/");
    if (ignoreSegments.length === 1) {
      const segment = ignoreSegments[0];
      if (segment.length === 0) continue;
      const pathSegments = relPath.split("/");
      if (pathSegments.includes(segment)) {
        return true;
      }
    }
  }
  return false;
}

function collectFiles(
  root: string,
  current: string,
  ignoreList: string[],
  files: string[]
): void {
  const relCurrent = normalizePath(relative(root, current));
  if (relCurrent && shouldIgnore(relCurrent, ignoreList)) {
    return;
  }

  let dirEntries;
  try {
    dirEntries = readdirSync(current, { withFileTypes: true });
  } catch (error) {
    console.warn(`Skipping directory due to read error (${current}): ${error}`);
    return;
  }

  for (const entry of dirEntries) {
    const fullPath = join(current, entry.name);
    const relPath = normalizePath(relative(root, fullPath));

    if (shouldIgnore(relPath, ignoreList)) {
      continue;
    }

    if (entry.isSymbolicLink()) {
      continue;
    }

    if (entry.isDirectory()) {
      collectFiles(root, fullPath, ignoreList, files);
    } else if (entry.isFile()) {
      files.push(fullPath);
    } else {
      continue;
    }
  }
}

function hashFile(path: string): string {
  const hash = createHash("sha256");
  const buffer = readFileSync(path);
  hash.update(buffer);
  return hash.digest("hex");
}

function isAllowedGroup(paths: string[], allowedGroups: string[][]): boolean {
  const normalized = paths.map((p) => normalizePath(p)).sort();
  return allowedGroups.some((group) => {
    const normalizedGroup = group.map((p) => normalizePath(p)).sort();
    if (normalizedGroup.length !== normalized.length) {
      return false;
    }
    return normalizedGroup.every((value, idx) => value === normalized[idx]);
  });
}

function main() {
  const repoRoot = getRepoRoot();
  const config = loadConfig(repoRoot);
  const files: string[] = [];
  collectFiles(repoRoot, repoRoot, config.ignore, files);

  const hashToPaths = new Map<string, string[]>();
  for (const file of files) {
    try {
      const stats = statSync(file);
      if (stats.size === 0) {
        continue;
      }
    } catch (error) {
      console.warn(`Skipping file due to stat error (${file}): ${error}`);
      continue;
    }

    const hash = hashFile(file);
    const list = hashToPaths.get(hash) ?? [];
    list.push(file);
    hashToPaths.set(hash, list);
  }

  const failures: string[] = [];
  for (const [hash, paths] of hashToPaths.entries()) {
    if (paths.length < 2) continue;
    const relPaths = paths.map((p) => normalizePath(relative(repoRoot, p)));
    if (!isAllowedGroup(relPaths, config.allowedDuplicateGroups)) {
      failures.push(
        `Hash ${hash} shared by multiple files without whitelist:\n${relPaths
          .map((p) => ` - ${p}`)
          .join("\n")}`
      );
    }
  }

  if (failures.length > 0) {
    console.error(
      "Duplicate content detected. Either deduplicate the files or register the exact set in tools/ci/check_duplicate_content.config.json."
    );
    console.error(failures.join("\n\n"));
    console.error(
      "If a one-time policy override is approved, apply the ci-allow-duplicate-content label to the pull request and include the approver in the summary."
    );
    process.exit(1);
  }

  console.log("Duplicate content check passed.");
}

main();
