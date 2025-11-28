import fs from 'node:fs';
import path from 'node:path';

export type NotebookSymbolChangeKind = 'added' | 'removed' | 'updated';

export interface NotebookSymbolMetadata {
  stable_id: string;
  name: string;
  kind: string;
  file: string;
  signature: string;
  span: [number, number];
}

export interface NotebookSymbolChange {
  stable_id: string;
  change: NotebookSymbolChangeKind;
  metadata?: NotebookSymbolMetadata | null;
}

export interface NotebookMetadataDiff {
  generated_at: string;
  changed: NotebookSymbolChange[];
}

export interface NotebookSyncResult {
  processedDiffs: number;
  notebooksTouched: number;
  processedFiles: string[];
  ledgerPath: string;
  analyticsPath: string;
}

interface NotebookFile {
  path: string;
  content: any;
  original: string;
}

interface AnalyticsLedger {
  runs: Array<{
    timestamp: string;
    processedDiffs: number;
    notebooksTouched: number;
    totalSymbolChanges: number;
  }>;
}

export function runNotebookSync(root: string = process.cwd()): NotebookSyncResult {
  const diffDir = path.join(root, '.workspace', 'notebook_sync', 'diffs');
  const ledgerPath = path.join(root, '.workspace', 'notebook_sync', 'ledger.json');
  const analyticsPath = path.join(root, 'metrics', 'notebook_analytics.json');
  const diffs = readDiffs(diffDir);

  if (diffs.active.length === 0 && diffs.all.length === 0) {
    ensureDirectory(path.dirname(ledgerPath));
    ensureDirectory(path.dirname(analyticsPath));
    appendAnalyticsRun(analyticsPath, {
      timestamp: new Date().toISOString(),
      processedDiffs: 0,
      notebooksTouched: 0,
      totalSymbolChanges: 0,
    });
    return {
      processedDiffs: 0,
      notebooksTouched: 0,
      processedFiles: [],
      ledgerPath,
      analyticsPath,
    };
  }

  const aggregatedChanges = aggregateChanges(diffs.active);

  let notebooksTouched = 0;
  for (const notebook of enumerateNotebooks(root)) {
    const changed = applyNotebookUpdates(notebook, aggregatedChanges, diffs.active);
    if (changed) {
      notebooksTouched += 1;
    }
    persistNotebook(notebook);
  }

  recordLedger(ledgerPath, diffs.active);
  appendAnalyticsRun(analyticsPath, {
    timestamp: new Date().toISOString(),
    processedDiffs: diffs.active.length,
    notebooksTouched,
    totalSymbolChanges: aggregatedChanges.length,
  });

  moveProcessedDiffs(diffDir, diffs.all);

  return {
    processedDiffs: diffs.active.length,
    notebooksTouched,
    processedFiles: diffs.all,
    ledgerPath,
    analyticsPath,
  };
}

function readDiffs(diffDir: string): { active: NotebookMetadataDiff[]; all: string[] } {
  if (!fs.existsSync(diffDir)) {
    return { active: [], all: [] };
  }
  const entries = fs.readdirSync(diffDir);
  const files = entries
    .map((entry) => path.join(diffDir, entry))
    .filter((item) => fs.statSync(item).isFile() && item.endsWith('.json'))
    .sort();
  const diffs: NotebookMetadataDiff[] = [];
  files.forEach((file) => {
    try {
      const raw = fs.readFileSync(file, 'utf8');
      const parsed = JSON.parse(raw) as NotebookMetadataDiff;
      if (Array.isArray(parsed.changed) && parsed.changed.length > 0) {
        diffs.push(parsed);
      }
    } catch (error) {
      console.warn(`[notebook-sync] failed to parse diff at ${file}:`, error);
    }
  });
  return { active: diffs, all: files };
}

function* enumerateNotebooks(root: string): Generator<NotebookFile> {
  const notebooksRoot = path.join(root, 'notebooks');
  if (!fs.existsSync(notebooksRoot)) {
    return;
  }
  for (const filePath of traverseDirectory(notebooksRoot)) {
    if (!filePath.endsWith('.ipynb')) {
      continue;
    }
    const original = fs.readFileSync(filePath, 'utf8');
    try {
      const content = JSON.parse(original);
      yield { path: filePath, content, original };
    } catch (error) {
      console.warn(`[notebook-sync] unable to parse notebook ${filePath}:`, error);
    }
  }
}

function* traverseDirectory(dir: string): Generator<string> {
  if (!fs.existsSync(dir)) {
    return;
  }
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.name === '.workspace' || entry.name === 'node_modules') {
      continue;
    }
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      yield* traverseDirectory(fullPath);
    } else if (entry.isFile()) {
      yield fullPath;
    }
  }
}

function aggregateChanges(diffs: NotebookMetadataDiff[]): NotebookSymbolChange[] {
  const merged = new Map<string, NotebookSymbolChange>();
  diffs.forEach((diff) => {
    diff.changed.forEach((change) => {
      merged.set(change.stable_id, change);
    });
  });
  return Array.from(merged.values());
}

function applyNotebookUpdates(
  notebook: NotebookFile,
  changes: NotebookSymbolChange[],
  diffs: NotebookMetadataDiff[],
): boolean {
  const metadata = (notebook.content.metadata = notebook.content.metadata || {});
  const noaMetadata = (metadata.noa = metadata.noa || {});
  const before = JSON.stringify(noaMetadata);
  noaMetadata.last_sync = new Date().toISOString();
  noaMetadata.diff_batches = diffs.map((diff) => diff.generated_at);
  noaMetadata.changed_symbols = changes.map((change) => ({
    stable_id: change.stable_id,
    change: change.change,
    name: change.metadata?.name ?? null,
    kind: change.metadata?.kind ?? null,
    file: change.metadata?.file ?? null,
    signature: change.metadata?.signature ?? null,
    span: change.metadata?.span ?? null,
  }));
  noaMetadata.citations = noaMetadata.changed_symbols
    .filter((entry: any) => entry.name && entry.file)
    .map((entry: any) => `${entry.name} · ${entry.file}`);
  const metadataChanged = JSON.stringify(noaMetadata) !== before;

  let cellsChanged = false;
  if (Array.isArray(notebook.content.cells)) {
    notebook.content.cells.forEach((cell: any) => {
      if (Array.isArray(cell.outputs) && cell.outputs.length > 0) {
        cell.outputs = [];
        cellsChanged = true;
      }
      if (cell.execution_count !== undefined && cell.execution_count !== null) {
        cell.execution_count = null;
        cellsChanged = true;
      }
      cell.metadata = cell.metadata || {};
      const cellBefore = JSON.stringify(cell.metadata.noa || {});
      cell.metadata.noa = {
        ...(cell.metadata.noa || {}),
        stripped_by: 'notebook-sync-agent',
        last_updated: noaMetadata.last_sync,
      };
      if (JSON.stringify(cell.metadata.noa) !== cellBefore) {
        cellsChanged = true;
      }
    });
  }

  return metadataChanged || cellsChanged;
}

function persistNotebook(notebook: NotebookFile): void {
  const serialised = JSON.stringify(notebook.content, null, 2);
  if (serialised !== notebook.original.trimEnd()) {
    fs.writeFileSync(notebook.path, `${serialised}\n`);
  }
}

function recordLedger(ledgerPath: string, diffs: NotebookMetadataDiff[]): void {
  ensureDirectory(path.dirname(ledgerPath));
  const existing = readJson<NotebookMetadataDiff[]>(ledgerPath, []);
  const next = existing.concat(diffs);
  fs.writeFileSync(ledgerPath, `${JSON.stringify(next, null, 2)}\n`);
}

function appendAnalyticsRun(analyticsPath: string, run: AnalyticsLedger['runs'][number]): void {
  ensureDirectory(path.dirname(analyticsPath));
  const ledger = readJson<AnalyticsLedger>(analyticsPath, { runs: [] });
  ledger.runs.push(run);
  fs.writeFileSync(analyticsPath, `${JSON.stringify(ledger, null, 2)}\n`);
}

function moveProcessedDiffs(diffDir: string, files: string[]): void {
  if (files.length === 0) {
    return;
  }
  const processedDir = path.join(diffDir, 'processed');
  ensureDirectory(processedDir);
  files.forEach((file) => {
    const destination = uniquePath(processedDir, path.basename(file));
    try {
      fs.renameSync(file, destination);
    } catch (error) {
      console.warn(`[notebook-sync] unable to move diff ${file}:`, error);
    }
  });
}

function uniquePath(dir: string, basename: string): string {
  let candidate = path.join(dir, basename);
  let counter = 1;
  while (fs.existsSync(candidate)) {
    const ext = path.extname(basename);
    const name = path.basename(basename, ext);
    candidate = path.join(dir, `${name}-${counter}${ext}`);
    counter += 1;
  }
  return candidate;
}

function ensureDirectory(dir: string): void {
  fs.mkdirSync(dir, { recursive: true });
}

function readJson<T>(file: string, fallback: T): T {
  if (!fs.existsSync(file)) {
    return fallback;
  }
  try {
    const raw = fs.readFileSync(file, 'utf8');
    return JSON.parse(raw) as T;
  } catch (error) {
    console.warn(`[notebook-sync] failed to read ${file}:`, error);
    return fallback;
  }
}
