import { readFileSync, readdirSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = new URL("../", import.meta.url);
const projectRoot = fileURLToPath(ROOT);
const allowedExtensions = new Set([".ts", ".tsx", ".js", ".json", ".css"]);
const issues = [];

function walk(directory) {
  const entries = readdirSync(directory, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = join(directory, entry.name);
    if (entry.isDirectory()) {
      // Skip build output folders if they appear.
      if (entry.name === "node_modules" || entry.name === ".next" || entry.name === "out") {
        continue;
      }
      walk(fullPath);
      continue;
    }

    const dotIndex = entry.name.lastIndexOf(".");
    const extension = dotIndex === -1 ? "" : entry.name.slice(dotIndex);
    if (!allowedExtensions.has(extension)) {
      continue;
    }

    const relativePath = relative(projectRoot, fullPath);
    const content = readFileSync(fullPath, "utf8");

    content.split(/\r?\n/).forEach((line, index) => {
      if (/\s+$/.test(line)) {
        issues.push(`${relativePath}:${index + 1} trailing whitespace`);
      }
    });

    if (extension === ".ts" || extension === ".tsx") {
      const todoMatches = content.match(/TODO|FIXME/g);
      if (todoMatches) {
        issues.push(`${relativePath} contains placeholder markers (TODO/FIXME)`);
      }
    }
  }
}

walk(projectRoot);

if (issues.length > 0) {
  console.error("Lint found issues:\n" + issues.map((issue) => ` - ${issue}`).join("\n"));
  process.exit(1);
}

console.log("✨ Offline lint checks passed");
