import { promises as fs } from "fs";
import { dirname, join } from "path";

import type { Workspace } from "../app/components/board-types";
import { logInfo, logWarn } from "@noa-ark/shared-ui/logging";

export type GoalExecutionTrace = {
  id: string;
  goalId: string;
  workspaceId: string;
  boardId?: string | null;
  actorId?: string | null;
  actorName?: string | null;
  action: string;
  summary?: string | null;
  metadata?: Record<string, unknown> | null;
  createdAt: string;
};

export type WorkspaceSnapshot = {
  workspaceId: string;
  capturedAt: string;
  boardCount: number;
  cardCount: number;
  activeMembers: number;
  notes?: string;
};

type GoalMemory = {
  goalId: string;
  workspaceId: string;
  traces: GoalExecutionTrace[];
};

type WorkspaceMemory = {
  workspaceId: string;
  snapshots: WorkspaceSnapshot[];
  goals: GoalMemory[];
};

type MemoryStore = {
  workspaces: WorkspaceMemory[];
};

const DATA_DIR = join(process.cwd(), ".data");
const MEMORY_FILE = join(DATA_DIR, "workspace-memory.json");
const MAX_TRACES_PER_GOAL = 200;
const MAX_SNAPSHOTS_PER_WORKSPACE = 32;

async function ensureStore(): Promise<void> {
  await fs.mkdir(dirname(MEMORY_FILE), { recursive: true });
  try {
    await fs.access(MEMORY_FILE);
  } catch {
    const initial: MemoryStore = { workspaces: [] };
    await fs.writeFile(MEMORY_FILE, JSON.stringify(initial, null, 2), "utf-8");
  }
}

async function readStore(): Promise<MemoryStore> {
  await ensureStore();
  const raw = await fs.readFile(MEMORY_FILE, "utf-8");
  return JSON.parse(raw) as MemoryStore;
}

async function writeStore(store: MemoryStore): Promise<void> {
  await ensureStore();
  await fs.writeFile(MEMORY_FILE, JSON.stringify(store, null, 2), "utf-8");
}

function findWorkspace(store: MemoryStore, workspaceId: string): WorkspaceMemory {
  let workspaceMemory = store.workspaces.find((entry) => entry.workspaceId === workspaceId);
  if (!workspaceMemory) {
    workspaceMemory = { workspaceId, snapshots: [], goals: [] };
    store.workspaces.push(workspaceMemory);
  }
  return workspaceMemory;
}

function findGoal(workspace: WorkspaceMemory, goalId: string): GoalMemory {
  let goalMemory = workspace.goals.find((entry) => entry.goalId === goalId);
  if (!goalMemory) {
    goalMemory = { goalId, workspaceId: workspace.workspaceId, traces: [] };
    workspace.goals.push(goalMemory);
  }
  return goalMemory;
}

export async function appendGoalTrace(trace: GoalExecutionTrace): Promise<void> {
  const store = await readStore();
  const workspace = findWorkspace(store, trace.workspaceId);
  const goal = findGoal(workspace, trace.goalId);
  goal.traces.unshift(trace);
  if (goal.traces.length > MAX_TRACES_PER_GOAL) {
    const removed = goal.traces.splice(MAX_TRACES_PER_GOAL);
    if (removed.length) {
      logWarn({
        component: "workspace.memory",
        event: "goal_trace_evicted",
        message: "Evicted goal execution traces to maintain retention policy",
        outcome: "evicted",
        context: {
          goalId: trace.goalId,
          workspaceId: trace.workspaceId,
          evicted: removed.length,
          retained: MAX_TRACES_PER_GOAL,
        },
      });
    }
  }
  await writeStore(store);
  logInfo({
    component: "workspace.memory",
    event: "goal_trace_recorded",
    message: `Recorded goal trace for ${trace.goalId}`,
    outcome: "success",
    context: {
      workspaceId: trace.workspaceId,
      boardId: trace.boardId ?? undefined,
      action: trace.action,
    },
  });
}

export async function listGoalTraces(goalId: string, workspaceId: string): Promise<GoalExecutionTrace[]> {
  const store = await readStore();
  const workspace = store.workspaces.find((entry) => entry.workspaceId === workspaceId);
  if (!workspace) {
    return [];
  }
  const goal = workspace.goals.find((entry) => entry.goalId === goalId);
  return goal ? goal.traces : [];
}

export async function summarizeGoalMemory(goalId: string, workspaceId: string) {
  const traces = await listGoalTraces(goalId, workspaceId);
  if (!traces.length) {
    return {
      goalId,
      workspaceId,
      traceCount: 0,
      lastSeen: null,
      summary: "No historical traces recorded yet.",
    };
  }
  const lastSeen = traces[0]?.createdAt ?? null;
  const actors = new Set(traces.map((trace) => trace.actorId).filter(Boolean));
  const actions = new Set(traces.map((trace) => trace.action));
  return {
    goalId,
    workspaceId,
    traceCount: traces.length,
    lastSeen,
    summary: `${traces.length} traces, ${actors.size} actors, ${actions.size} actions recorded.`,
  };
}

export async function recordWorkspaceSnapshot(workspace: Workspace): Promise<void> {
  const store = await readStore();
  const workspaceMemory = findWorkspace(store, workspace.id);
  const cardCount = workspace.boards.reduce(
    (count, board) => count + board.columns.reduce((acc, column) => acc + column.cards.length, 0),
    0
  );
  const snapshot: WorkspaceSnapshot = {
    workspaceId: workspace.id,
    capturedAt: new Date().toISOString(),
    boardCount: workspace.boards.length,
    cardCount,
    activeMembers: workspace.members.length,
    notes: workspace.lastSyncedAt ? `Last synced ${workspace.lastSyncedAt}` : undefined,
  };
  workspaceMemory.snapshots.unshift(snapshot);
  if (workspaceMemory.snapshots.length > MAX_SNAPSHOTS_PER_WORKSPACE) {
    const removed = workspaceMemory.snapshots.splice(MAX_SNAPSHOTS_PER_WORKSPACE);
    if (removed.length) {
      logWarn({
        component: "workspace.memory",
        event: "workspace_snapshot_evicted",
        message: "Evicted workspace snapshots to maintain retention policy",
        outcome: "evicted",
        context: {
          workspaceId: workspace.id,
          evicted: removed.length,
          retained: MAX_SNAPSHOTS_PER_WORKSPACE,
        },
      });
    }
  }
  try {
    await writeStore(store);
    logInfo({
      component: "workspace.memory",
      event: "workspace_snapshot_recorded",
      message: `Snapshot recorded for workspace ${workspace.id}`,
      outcome: "success",
      context: {
        boardCount: snapshot.boardCount,
        cardCount: snapshot.cardCount,
      },
    });
  } catch (error) {
    logWarn({
      component: "workspace.memory",
      event: "workspace_snapshot_write_failed",
      message: `Failed to write workspace snapshot for workspace ${workspace.id}`,
      outcome: "error",
      context: {
        error: error instanceof Error ? error.message : String(error),
        workspaceId: workspace.id,
      },
    });
    throw error;
  }

export async function getWorkspaceSnapshots(workspaceId: string): Promise<WorkspaceSnapshot[]> {
  const store = await readStore();
  const workspace = store.workspaces.find((entry) => entry.workspaceId === workspaceId);
  return workspace ? workspace.snapshots : [];
}

export async function getGoalMemoryInsights(goalId: string, workspaceId: string) {
  const summary = await summarizeGoalMemory(goalId, workspaceId);
  const traces = await listGoalTraces(goalId, workspaceId);
  return {
    ...summary,
    traces: traces.slice(0, 20),
  };
}
