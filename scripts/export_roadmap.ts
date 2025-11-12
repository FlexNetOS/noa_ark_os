#!/usr/bin/env node
/**
 * Export the PM roadmap markdown into CSV and JSON build kits.
 * This script is invoked from package scripts and parses docs/plans/roadmap.md.
 */

import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const ROADMAP_PATH = resolve("docs/plans/roadmap.md");
const CSV_PATH = resolve("build_kits/pm_roadmap.csv");
const JSON_PATH = resolve("build_kits/pm_roadmap.json");

const CSV_HEADER = "code,title,theme,description,priority,status,depends_on,acceptance_criteria";

async function loadRoadmapContent() {
  const raw = await readFile(ROADMAP_PATH, "utf8");
  const markerStart = "<!-- BEGIN: PM_ROADMAP -->";
  const markerEnd = "<!-- END: PM_ROADMAP -->";
  const startIndex = raw.indexOf(markerStart);
  const endIndex = raw.indexOf(markerEnd);
  if (startIndex === -1 || endIndex === -1 || endIndex <= startIndex) {
    throw new Error("PM_ROADMAP markers not found in roadmap.md");
  }
  return raw.slice(startIndex + markerStart.length, endIndex).trim();
}

function extractSummaryAnchors(content: string) {
  const lines = content.split(/\r?\n/);
  const anchors: { theme: string; anchor: string }[] = [];
  let currentTheme: string | null = null;

  for (const line of lines) {
    const themeMatch = /^### \d+\.\s*(.+)$/.exec(line.trim());
    if (themeMatch) {
      currentTheme = themeMatch[1].trim();
      continue;
    }
    if (!currentTheme) {
      continue;
    }
    const viewMatch = /\[View task\]\(#([^)]+)\)/.exec(line);
    if (viewMatch) {
      anchors.push({ theme: currentTheme, anchor: viewMatch[1] });
      currentTheme = null;
    }
  }

  if (anchors.length === 0) {
    throw new Error("No summary anchors found in roadmap content");
  }

  return anchors;
}

interface TaskDetail {
  code: string;
  title: string;
  description: string;
  checklist: string[];
  acceptanceCriteria: string[];
  priority: string;
  status: string;
  dependsOn: string[];
}

function extractTaskDetails(content: string): Record<string, TaskDetail> {
  const details: Record<string, TaskDetail> = {};
  /**
   * Regex to extract each task block from the roadmap markdown.
   * Capture groups:
   *   1. anchor         - The HTML anchor id (e.g., "agentos-001")
   *   2. code           - The task code (e.g., "AGENTOS-001")
   *   3. title          - The task title (text after the em dash)
   *   4. description    - The description text (single line)
   *   5. checklistBlock - The checklist block (multi-line, up to Acceptance criteria)
   *   6. acceptanceBlock- The acceptance criteria block (multi-line, up to Meta)
   *   7. metaBlock      - The meta block (multi-line, up to next anchor or end of content)
   */
  const taskRegex = /<a id="([^"]+)"><\/a>\s*###\s*(AGENTOS-\d+)\s*—\s*([^\n]+)\n\*\*Description:\*\*\s*([^\n]+)\n\*\*Checklist\*\*\n([\s\S]*?)\n\*\*Acceptance criteria\*\*\n([\s\S]*?)\n\*\*Meta\*\*\n([\s\S]*?)(?=\n<a id=|$)/g;

  let match: RegExpExecArray | null;
  while ((match = taskRegex.exec(content)) !== null) {
    const [, anchor, code, title, description, checklistBlock, acceptanceBlock, metaBlock] = match;
    const checklist = checklistBlock
      .split(/\r?\n/)
      .map((line) => line.replace(/^[-*]\s*\[.\]\s*/, "").trim())
      .filter((line) => line.length > 0);
    const acceptanceCriteria = acceptanceBlock
      .split(/\r?\n/)
      .map((line) => line.replace(/^[-*]\s*/, "").trim())
      .filter((line) => line.length > 0);

    const priorityMatch = /Priority:\s*(.+)/.exec(metaBlock);
    const statusMatch = /Status:\s*(.+)/.exec(metaBlock);
    const dependsMatch = /Depends on:\s*(.+)/.exec(metaBlock);
    const priority = priorityMatch ? priorityMatch[1].trim() : "";
    const status = statusMatch ? statusMatch[1].trim() : "";
    const dependsRaw = dependsMatch ? dependsMatch[1].trim() : "";
    const dependsOn = dependsRaw && dependsRaw.toLowerCase() !== "none" ? dependsRaw.split(/,\s*/) : [];

    details[anchor] = {
      code,
      title,
      description,
      checklist,
      acceptanceCriteria,
      priority,
      status,
      dependsOn,
    };
  }

  if (Object.keys(details).length === 0) {
    throw new Error("No task details parsed from roadmap content");
  }

  return details;
}

function escapeCsvCell(value: string) {
  if (value === "") {
    return value;
  }
  const needsQuotes = /[",\n]/.test(value);
  if (needsQuotes) {
    return `"${value.replace(/"/g, '""')}"`;
  }
  return value;
}

function toCsv(tasks: (TaskDetail & { theme: string })[]) {
  const rows = tasks.map((task) => {
    const acceptanceCell = escapeCsvCell(task.acceptanceCriteria.join("|;"));
    const dependsCell = escapeCsvCell(task.dependsOn.join("|;"));
    return [
      escapeCsvCell(task.code),
      escapeCsvCell(task.title),
      escapeCsvCell(task.theme),
      escapeCsvCell(task.description),
      escapeCsvCell(task.priority),
      escapeCsvCell(task.status),
      dependsCell,
      acceptanceCell,
    ].join(",");
  });
  return [CSV_HEADER, ...rows].join("\n");
}

function toJson(tasks: (TaskDetail & { theme: string })[]) {
  const payload = tasks.map((task) => ({
    code: task.code,
    title: task.title,
    theme: task.theme,
    description: task.description,
    priority: task.priority,
    status: task.status,
    depends_on: task.dependsOn,
    acceptance_criteria: task.acceptanceCriteria,
  }));
  return JSON.stringify(payload, null, 2) + "\n";
}

async function main() {
  const content = await loadRoadmapContent();
  const anchors = extractSummaryAnchors(content);
  const details = extractTaskDetails(content);

  const orderedTasks = anchors.map(({ theme, anchor }) => {
    const detail = details[anchor];
    if (!detail) {
      throw new Error(`Anchor ${anchor} referenced in summary but missing in task details`);
    }
    return { ...detail, theme };
  });

  await writeFile(CSV_PATH, toCsv(orderedTasks));
  await writeFile(JSON_PATH, toJson(orderedTasks));
}

main().catch((error) => {
  console.error("Failed to export roadmap:", error);
  process.exitCode = 1;
});
