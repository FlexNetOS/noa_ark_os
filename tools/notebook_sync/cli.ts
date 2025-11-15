#!/usr/bin/env node
import path from 'node:path';
import process from 'node:process';

import { runNotebookSync } from './agent.ts';

function resolveRoot(args: string[]): string {
  for (const arg of args) {
    if (!arg.startsWith('-')) {
      return path.resolve(process.cwd(), arg);
    }
  }
  return process.cwd();
}

function main(): void {
  const root = resolveRoot(process.argv.slice(2));
  const result = runNotebookSync(root);
  process.stdout.write(
    `notebook-sync: processed ${result.processedDiffs} diffs across ${result.notebooksTouched} notebooks\n`,
  );
  if (result.processedFiles.length > 0) {
    process.stdout.write(
      `notebook-sync: archived diff files:\n${result.processedFiles.map((file) => `  - ${file}`).join('\n')}\n`,
    );
  }
}

main();
