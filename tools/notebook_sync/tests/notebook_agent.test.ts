import test from 'node:test';
import assert from 'node:assert';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { runNotebookSync, NotebookMetadataDiff } from '../agent.ts';

test('runNotebookSync refreshes notebook metadata and analytics', () => {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), 'notebook-sync-'));
  const diffDir = path.join(workspace, '.workspace', 'notebook_sync', 'diffs');
  fs.mkdirSync(diffDir, { recursive: true });

  const diff: NotebookMetadataDiff = {
    generated_at: '2024-06-01T00:00:00Z',
    changed: [
      {
        stable_id: 'id::demo',
        change: 'added',
        metadata: {
          stable_id: 'id::demo',
          name: 'demo',
          kind: 'function',
          file: 'src/lib.rs',
          signature: 'fn demo()',
          span: [1, 2],
        },
      },
    ],
  };
  fs.writeFileSync(path.join(diffDir, 'diff-1.json'), JSON.stringify(diff, null, 2));

  const notebookDir = path.join(workspace, 'notebooks');
  fs.mkdirSync(notebookDir, { recursive: true });
  const notebookPath = path.join(notebookDir, 'example.ipynb');
  const notebook = {
    metadata: {},
    cells: [
      {
        cell_type: 'code',
        source: ['print("hello")'],
        outputs: [
          {
            output_type: 'stream',
            text: ['hello'],
          },
        ],
        execution_count: 1,
        metadata: {},
      },
    ],
    nbformat: 4,
    nbformat_minor: 5,
  };
  fs.writeFileSync(notebookPath, JSON.stringify(notebook, null, 2));

  const result = runNotebookSync(workspace);
  assert.strictEqual(result.processedDiffs, 1);
  assert.strictEqual(result.notebooksTouched, 1);

  const updated = JSON.parse(fs.readFileSync(notebookPath, 'utf8'));
  assert.ok(Array.isArray(updated.cells));
  assert.deepStrictEqual(updated.cells[0].outputs, []);
  assert.strictEqual(updated.cells[0].execution_count, null);
  assert.ok(updated.metadata?.noa);
  assert.strictEqual(updated.metadata.noa.changed_symbols[0].stable_id, 'id::demo');

  const ledgerPath = path.join(workspace, '.workspace', 'notebook_sync', 'ledger.json');
  const ledger = JSON.parse(fs.readFileSync(ledgerPath, 'utf8'));
  assert.strictEqual(Array.isArray(ledger), true);
  assert.strictEqual(ledger.length, 1);

  const analyticsPath = path.join(workspace, 'metrics', 'notebook_analytics.json');
  const analytics = JSON.parse(fs.readFileSync(analyticsPath, 'utf8'));
  assert.ok(Array.isArray(analytics.runs));
  assert.strictEqual(analytics.runs.at(-1)?.processedDiffs, 1);
});
