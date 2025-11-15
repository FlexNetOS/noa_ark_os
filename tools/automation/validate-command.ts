#!/usr/bin/env node
import process from "node:process";
import { evaluateCommand } from "./command_policy.ts";

function printUsage(): void {
  process.stderr.write(
    "Usage: validate-command.ts [--policy <path>] [--explain] <command> [args...]\n",
  );
}

function main(): void {
  const args = process.argv.slice(2);
  let policyPath: string | undefined;
  let explain = false;
  const command: string[] = [];

  for (let i = 0; i < args.length; i += 1) {
    const arg = args[i];
    if (arg === "--policy") {
      if (i + 1 >= args.length) {
        process.stderr.write("--policy requires a path\n");
        process.exitCode = 2;
        return;
      }
      policyPath = args[i + 1];
      i += 1;
      continue;
    }
    if (arg === "--explain") {
      explain = true;
      continue;
    }
    command.push(arg);
  }

  if (command.length === 0) {
    printUsage();
    process.exitCode = 2;
    return;
  }

  try {
    const evaluation = evaluateCommand(command, { policyPath });
    if (!evaluation.allowed) {
      const ruleHint = evaluation.rule ? ` (rule: ${evaluation.rule.id})` : "";
      const reason = evaluation.reason ? ` - ${evaluation.reason}` : "";
      process.stderr.write(`Command blocked${ruleHint}: ${command.join(" ")}${reason}\n`);
      process.exitCode = 1;
      return;
    }
    if (explain) {
      const ruleHint = evaluation.rule ? evaluation.rule.id : "default-allow";
      const reason = evaluation.reason ? `: ${evaluation.reason}` : "";
      process.stdout.write(`allowed by ${ruleHint}${reason}\n`);
    }
  } catch (error) {
    process.stderr.write(`validator error: ${(error as Error).message}\n`);
    process.exitCode = 3;
  }
}

main();
