import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import process from "node:process";

interface PointerCheck {
  readonly path: string;
  readonly expected: string;
}

const repoRoot = resolve(process.cwd());

const checks: PointerCheck[] = [
  {
    path: ".copilot",
    expected: "# Defer to AGENT.md\nUse AGENT.md at repo root as the sole policy and instruction source. Do not duplicate logic here.\n",
  },
  {
    path: "CLAUDE.md",
    expected: "This file intentionally contains no instructions. All policies and execution rules are defined in AGENT.md.\n",
  },
  {
    path: ".github/AGENT_POINTER.txt",
    expected: "Provider/agent instruction policy lives in AGENT.md. Do not place instructions elsewhere.\n",
  },
];

let hasError = false;

for (const check of checks) {
  const targetPath = resolve(repoRoot, check.path);
  let fileContent: string;
  try {
    fileContent = readFileSync(targetPath, "utf8");
  } catch (error) {
    process.stderr.write(`Missing required provider pointer: ${check.path}\n`);
    hasError = true;
    continue;
  }

  const normalized = fileContent.replace(/\r\n/g, "\n");
  const expected = check.expected;

  if (normalized !== expected) {
    process.stderr.write(
      `Provider pointer mismatch for ${check.path}.` +
        "\n" +
        `Expected:\n${expected}` +
        "\n" +
        `Received:\n${normalized}`
    );
    hasError = true;
  }
}

if (hasError) {
  process.stderr.write("Provider pointer verification failed.\n");
  process.exit(1);
}

process.stdout.write("Provider pointer verification passed.\n");
