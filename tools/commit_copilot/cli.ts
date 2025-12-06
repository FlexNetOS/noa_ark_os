#!/usr/bin/env node
import process from "node:process";
import { enforceLatestCommit, isConventionalCommit, suggestCommitMessage } from "./conventional.ts";

function usage(): void {
  process.stderr.write("Usage: commit-copilot <enforce|check|suggest> [--message=<value>]\n");
}

function parseArgs(argv: string[]): Record<string, string> {
  const args: Record<string, string> = {};
  for (const arg of argv) {
    if (arg.startsWith("--")) {
      const [key, value] = arg.slice(2).split("=", 2);
      if (value) {
        args[key] = value;
      }
    }
  }
  return args;
}

async function main() {
  const [, , command, ...rest] = process.argv;
  try {
    switch (command) {
      case "enforce":
        enforceLatestCommit();
        process.stdout.write("commit message ok\n");
        break;
      case "check": {
        const options = parseArgs(rest);
        const message = options.message ?? "";
        const valid = isConventionalCommit(message);
        process.stdout.write(JSON.stringify({ valid }) + "\n");
        process.exitCode = valid ? 0 : 1;
        break;
      }
      case "suggest": {
        const suggestion = await suggestCommitMessage();
        process.stdout.write(suggestion + "\n");
        break;
      }
      default:
        usage();
        process.exitCode = 1;
    }
  } catch (error) {
    process.stderr.write(`commit copilot error: ${(error as Error).message}\n`);
    process.exitCode = 1;
  }
}

void main();
