#!/usr/bin/env node
import process from "node:process";
import { createPr, recordReview, runChecksAndMaybeMerge, runAiReview, listTriageTickets } from "./lib/queue.ts";
import { readJson } from "./lib/fs.ts";
import { prFile, reviewFile } from "./lib/state.ts";

function parseArgs(argv: string[]): Record<string, string | boolean> {
  const args: Record<string, string | boolean> = {};
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg.startsWith("--")) {
      const key = arg.slice(2);
      const next = argv[i + 1];
      if (next && !next.startsWith("--")) {
        args[key] = next;
        i++; // skip the value
      } else {
        args[key] = true;
      }
    }
  }
  return args;
}

async function handleCreate(argv: string[]): Promise<void> {
  const options = parseArgs(argv);
  const pr = createPr({
    title: typeof options.title === "string" ? options.title : undefined,
    branch: typeof options.branch === "string" ? options.branch : undefined,
    base: typeof options.base === "string" ? options.base : undefined,
  });
  process.stdout.write(JSON.stringify(pr, null, 2) + "\n");
}

async function handleReview(argv: string[]): Promise<void> {
  const options = parseArgs(argv);
  const id = Number(options.id);
  if (!Number.isFinite(id)) {
    throw new Error("--id is required for review");
  }
  const reviewer = typeof options.reviewer === "string" ? options.reviewer : "offline-reviewer";
  const state = typeof options.state === "string" ? options.state : "comment";
  const body = typeof options.body === "string" ? options.body : "";
  const entry = recordReview({ id, reviewer, state, body });
  process.stdout.write(JSON.stringify(entry, null, 2) + "\n");
}

async function handleAiReview(argv: string[]): Promise<void> {
  const options = parseArgs(argv);
  const id = Number(options.id);
  if (!Number.isFinite(id)) {
    throw new Error("--id is required for ai-review");
  }
  const review = await runAiReview(id);
  process.stdout.write(JSON.stringify(review, null, 2) + "\n");
}

async function handleMerge(argv: string[]): Promise<void> {
  const options = parseArgs(argv);
  const id = Number(options.id);
  if (!Number.isFinite(id)) {
    throw new Error("--id is required for merge");
  }
  const autoApplyAi = options["no-ai"] ? false : true;
  const result = await runChecksAndMaybeMerge(id, { autoApplyAi });
  process.stdout.write(JSON.stringify(result, null, 2) + "\n");
}

async function handleShow(argv: string[]): Promise<void> {
  const options = parseArgs(argv);
  const id = Number(options.id);
  if (!Number.isFinite(id)) {
    throw new Error("--id is required for show");
  }
  const pr = readJson(prFile(id));
  const reviews = readJson(reviewFile(id), []);
  process.stdout.write(JSON.stringify({ pr, reviews }, null, 2) + "\n");
}

async function handleTriage(): Promise<void> {
  const triage = listTriageTickets();
  process.stdout.write(JSON.stringify(triage, null, 2) + "\n");
}

async function main() {
  const [, , command, ...rest] = process.argv;
  try {
    switch (command) {
      case "create":
        await handleCreate(rest);
        break;
      case "review":
        await handleReview(rest);
        break;
      case "ai-review":
        await handleAiReview(rest);
        break;
      case "merge":
        await handleMerge(rest);
        break;
      case "show":
        await handleShow(rest);
        break;
      case "triage":
        await handleTriage();
        break;
      default:
        process.stderr.write(`Unknown command: ${command}\n`);
        process.exitCode = 1;
        return;
    }
  } catch (error) {
    process.stderr.write(`offline queue error: ${(error as Error).message}\n`);
    process.exitCode = 1;
  }
}

void main();
