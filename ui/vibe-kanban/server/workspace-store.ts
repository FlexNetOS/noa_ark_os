import { promises as fs } from "fs";
import path from "path";

import { recordWorkspaceSnapshot } from "./memory-store";
import { workspaceEventHub } from "./workspace-events";

import type {
  ActivityEvent,
  AgentAutomationRun,
  GoalAutomationState,
  Goal,
  NotificationEvent,
  ToolExecutionTelemetry,
  UploadReceiptSummary,
  VibeColumn,
  Workspace,
  WorkspaceBoard,
  WorkspaceMember,
  BoardMetrics,
} from "../app/components/board-types";
import { findGoalMetric } from "./goal-analytics";

const DATA_DIR = path.join(process.cwd(), "data");
const DATA_FILE = path.join(DATA_DIR, "workspaces.json");

type WorkspaceStore = {
  workspaces: Workspace[];
};

function ensureAutomationState(
  cardId: string,
  automation: GoalAutomationState | null | undefined,
  fallbackDate: string
): GoalAutomationState | null {
  if (automation === null) {
    return null;
  }
  if (!automation) {
    return {
      goalId: cardId,
      history: [],
      lastUpdated: fallbackDate,
      retryAvailable: true,
    };
  }
  return {
    goalId: automation.goalId ?? cardId,
    history: Array.isArray(automation.history) ? automation.history : [],
    lastUpdated: automation.lastUpdated ?? fallbackDate,
    retryAvailable: automation.retryAvailable ?? true,
  };
}

function getColumnGoals(column: VibeColumn | { cards?: Goal[]; goals?: Goal[] }): Goal[] {
  if (Array.isArray((column as { goals?: Goal[] }).goals)) {
    return ((column as { goals?: Goal[] }).goals ?? []).map((goal) => ({ ...goal }));
  }
  if (Array.isArray((column as { cards?: Goal[] }).cards)) {
    return ((column as { cards?: Goal[] }).cards ?? []).map((goal) => ({ ...goal }));
  }
  return [];
}

function normaliseBoard(board: WorkspaceBoard): WorkspaceBoard {
  return {
    ...board,
    columns: board.columns.map((column) => {
      const goals = getColumnGoals(column).map((goal) => ({
        ...goal,
        automation: ensureAutomationState(
          goal.id,
          goal.automation as GoalAutomationState | null | undefined,
          goal.createdAt ?? new Date().toISOString()
        ),
      }));
      return {
        ...column,
        goals,
        cards: undefined,
      };
    }),
  };
}

async function ensureDataFile(): Promise<void> {
  await fs.mkdir(DATA_DIR, { recursive: true });
  try {
    await fs.access(DATA_FILE);
  } catch {
    const now = new Date().toISOString();
    const defaultWorkspace: Workspace = {
      id: "studio",
      name: "Vibe Studio",
      accent: "from-indigo-500 via-purple-500 to-blue-500",
      createdAt: now,
      billingPlan: "starter",
      members: [
        { id: "ava", name: "Ava", role: "owner", avatarHue: 265 },
        { id: "kai", name: "Kai", role: "member", avatarHue: 190 },
        { id: "sol", name: "Sol", role: "member", avatarHue: 20 }
      ],
      boards: [
        {
          id: "launchpad",
          workspaceId: "studio",
          goalId: "launchpad",
          projectName: "Vibe Coding Launchpad",
          description: "Ship the new kanban workspace with presence-aware collaboration.",
          lastUpdated: now,
          accent: "from-indigo-500 via-purple-500 to-blue-500",
          columns: [
            {
              id: "todo",
              title: "To Do",
              accent: "from-indigo-500 via-purple-500 to-blue-500",
              goals: [
                {
                  id: "goal-1",
                  title: "Ideate hero motion",
                  notes: "Sketch the flowing hero animation that loops smoothly across the dashboard.",
                  createdAt: now,
                  mood: "flow",
                  automation: ensureAutomationState("card-1", null, now),
                },
                {
                  id: "goal-2",
                  title: "Sound-reactive palette",
                  notes: "Research how to tie track BPM to gradient shifts on the home screen.",
                  createdAt: now,
                  mood: "focus",
                  automation: ensureAutomationState("card-2", null, now),
                }
              ]
            },
            {
              id: "in-progress",
              title: "In Progress",
              accent: "from-sky-500 via-cyan-400 to-emerald-400",
              goals: [
                {
                  id: "goal-3",
                  title: "Prototype kanban drag",
                  notes: "Polish easing curve + inertia for drag transitions.",
                  createdAt: now,
                  mood: "hype",
                  automation: ensureAutomationState("card-3", null, now),
                }
              ]
            },
            {
              id: "done",
              title: "Completed",
              accent: "from-violet-500 via-indigo-400 to-fuchsia-500",
              goals: [
                {
                  id: "goal-4",
                  title: "Mood-driven theme",
                  notes: "Shipped gradient system that syncs with vibes.",
                  createdAt: now,
                  mood: "chill",
                  automation: ensureAutomationState("card-4", null, now),
                }
              ]
            }
          ],
          metrics: {
            completedGoals: 1,
            activeGoals: 3,
            goalMomentum: 72
          },
          archived: false,
          moodSamples: [
            { recordedAt: now, focus: 0.4, flow: 0.3, chill: 0.1, hype: 0.2 }
          ]
        }
      ],
      activity: [
        {
          id: "act-1",
          type: "board.created",
          actorId: "ava",
          actorName: "Ava",
          boardId: "launchpad",
          description: "Ava spawned the Launchpad board",
          createdAt: now
        }
      ],
      notifications: [],
      uploadReceipts: []
    };

    const initial: WorkspaceStore = { workspaces: [defaultWorkspace] };
    await fs.writeFile(DATA_FILE, JSON.stringify(initial, null, 2), "utf-8");
  }
}

async function readStore(): Promise<WorkspaceStore> {
  await ensureDataFile();
  const raw = await fs.readFile(DATA_FILE, "utf-8");
  const parsed = JSON.parse(raw) as WorkspaceStore;
  const migrateBoard = async (board: WorkspaceBoard): Promise<WorkspaceBoard> => {
    const migratedColumns: VibeColumn[] = board.columns.map((column) => ({
      ...column,
      goals: getColumnGoals(column),
    }));

    const baseMetrics =
      board.metrics ??
      (await computeBoardMetrics({
        id: board.id,
        goalId: board.goalId,
        columns: migratedColumns,
      }));
    const normalizedMetrics: BoardMetrics = {
      completedGoals: baseMetrics.completedGoals ?? (baseMetrics as { completedCards?: number }).completedCards ?? 0,
      activeGoals: baseMetrics.activeGoals ?? (baseMetrics as { activeCards?: number }).activeCards ?? 0,
      goalMomentum: baseMetrics.goalMomentum ?? (baseMetrics as { vibeMomentum?: number }).vibeMomentum ?? 0,
      cycleTimeDays: baseMetrics.cycleTimeDays,
      flowEfficiency: baseMetrics.flowEfficiency,
    };

    return normaliseBoard({
      ...board,
      columns: migratedColumns,
      metrics: normalizedMetrics,
    });
  };

  return {
    workspaces: await Promise.all(
      parsed.workspaces.map(async (workspace) => ({
        ...workspace,
        boards: await Promise.all(workspace.boards.map((board) => migrateBoard(board as WorkspaceBoard))),
        notifications: workspace.notifications ?? [],
        uploadReceipts: workspace.uploadReceipts ?? [],
      }))
    ),
  };
}

async function writeStore(store: WorkspaceStore): Promise<void> {
  await ensureDataFile();
  await fs.writeFile(DATA_FILE, JSON.stringify(store, null, 2), "utf-8");
  const snapshotResults = await Promise.allSettled(
    store.workspaces.map((workspace) => recordWorkspaceSnapshot(workspace))
  );
  snapshotResults.forEach((result, idx) => {
    if (result.status === "rejected") {
      console.error(
        `Failed to record snapshot for workspace "${store.workspaces[idx]?.id ?? idx}":`,
        result.reason
      );
    }
  });
}

function getInMemoryStore(): { data: WorkspaceStore } {
  const globalAny = globalThis as typeof globalThis & { __workspaceStore?: { data: WorkspaceStore } };
  if (!globalAny.__workspaceStore) {
    globalAny.__workspaceStore = { data: { workspaces: [] } };
  }
  return globalAny.__workspaceStore;
}

async function hydrateStore(): Promise<WorkspaceStore> {
  const holder = getInMemoryStore();
  if (!holder.data.workspaces.length) {
    holder.data = await readStore();
  }
  return holder.data;
}

export async function listWorkspacesForUser(userId: string): Promise<Workspace[]> {
  const store = await hydrateStore();
  return store.workspaces.filter((workspace) => workspace.members.some((member) => member.id === userId));
}

export async function getWorkspace(workspaceId: string): Promise<Workspace | undefined> {
  const store = await hydrateStore();
  return store.workspaces.find((workspace) => workspace.id === workspaceId);
}

export async function upsertWorkspace(workspace: Workspace): Promise<Workspace> {
  const store = await hydrateStore();
  const existingIndex = store.workspaces.findIndex((item) => item.id === workspace.id);
  if (existingIndex === -1) {
    store.workspaces.push(workspace);
  } else {
    store.workspaces[existingIndex] = workspace;
  }
  await writeStore(store);
  return workspace;
}

export async function getBoard(workspaceId: string, boardId: string): Promise<WorkspaceBoard | undefined> {
  const workspace = await getWorkspace(workspaceId);
  return workspace?.boards.find((board) => board.id === boardId);
}

export async function saveBoard(
  workspaceId: string,
  nextBoard: WorkspaceBoard,
  actor: WorkspaceMember
): Promise<{ board: WorkspaceBoard; activity: ActivityEvent }> {
  const workspace = await getWorkspace(workspaceId);
  if (!workspace) {
    throw new Error(`Workspace ${workspaceId} not found`);
  }
  const boardIndex = workspace.boards.findIndex((board) => board.id === nextBoard.id);
  let normalisedBoard = normaliseBoard(nextBoard);
  normalisedBoard = {
    ...normalisedBoard,
    metrics: await computeBoardMetrics(normalisedBoard),
  };
  if (boardIndex === -1) {
    workspace.boards.push(normalisedBoard);
  } else {
    workspace.boards[boardIndex] = normalisedBoard;
  }

  const activity: ActivityEvent = {
    id: `act-${Date.now()}`,
    type: "board.updated",
    actorId: actor.id,
    actorName: actor.name,
    boardId: normalisedBoard.id,
    description: `${actor.name} synced ${normalisedBoard.projectName}`,
    createdAt: new Date().toISOString(),
  };
  workspace.activity.unshift(activity);
  workspace.activity = workspace.activity.slice(0, 50);

  const goalId = nextBoard.goalId ?? nextBoard.id;
  nextBoard.goalId = goalId;
  nextBoard.metrics = await computeBoardMetrics({
    columns: nextBoard.columns,
    goalId,
    id: nextBoard.id,
  });

  await upsertWorkspace({ ...workspace, lastSyncedAt: new Date().toISOString() } as Workspace);
  return { board: normalisedBoard, activity };
}

export async function recordUploadReceipt(
  workspaceId: string,
  receipt: Omit<UploadReceiptSummary, "id" | "workspaceId"> & { id?: string }
): Promise<UploadReceiptSummary> {
  const workspace = await getWorkspace(workspaceId);
  if (!workspace) {
    throw new Error(`Workspace ${workspaceId} not found`);
  }
  const entry: UploadReceiptSummary = {
    id: receipt.id ?? `upload-${Date.now()}`,
    workspaceId,
    boardId: receipt.boardId,
    dropId: receipt.dropId,
    dropType: receipt.dropType,
    originalName: receipt.originalName,
    casKeys: receipt.casKeys,
    receiptPath: receipt.receiptPath,
    uploadedAt: receipt.uploadedAt,
    uploadedBy: receipt.uploadedBy,
  };
  workspace.uploadReceipts = [entry, ...workspace.uploadReceipts].slice(0, 50);
  await upsertWorkspace(workspace);
  return entry;
}

export async function recordWorkspaceNotification(
  workspaceId: string,
  notification: NotificationEvent
): Promise<NotificationEvent> {
  const workspace = await getWorkspace(workspaceId);
  if (!workspace) {
    throw new Error(`Workspace ${workspaceId} not found`);
  }
  workspace.notifications = [notification, ...workspace.notifications].slice(0, 50);
  await upsertWorkspace(workspace);
  return notification;
}

type AutomationUpdate = {
  agentId: string;
  agentName: string;
  status: AgentAutomationRun["status"];
  toolResults: ToolExecutionTelemetry[];
  notes?: string;
  attempt?: number;
  occurredAt?: string;
};

export async function recordGoalAutomationProgress(
  workspaceId: string,
  boardId: string,
  cardId: string,
  update: AutomationUpdate
): Promise<{ automation: GoalAutomationState; activity: ActivityEvent }> {
  const workspace = await getWorkspace(workspaceId);
  if (!workspace) {
    throw new Error(`Workspace ${workspaceId} not found`);
  }
  const board = workspace.boards.find((entry) => entry.id === boardId);
  if (!board) {
    throw new Error(`Board ${boardId} not found in workspace ${workspaceId}`);
  }

  const column = board.columns.find((entry) => getColumnGoals(entry).some((card) => card.id === cardId));
  if (!column) {
    throw new Error(`Card ${cardId} not found in workspace ${workspaceId}`);
  }

  const card = getColumnGoals(column).find((entry) => entry.id === cardId);
  if (!card) {
    throw new Error(`Card ${cardId} not found`);
  }

  const timestamp = update.occurredAt ?? new Date().toISOString();
  const automation = ensureAutomationState(cardId, card.automation ?? undefined, timestamp) ?? {
    goalId: cardId,
    history: [],
    lastUpdated: timestamp,
    retryAvailable: true,
  };

  const run: AgentAutomationRun = {
    agentId: update.agentId,
    agentName: update.agentName,
    status: update.status,
    attempt: update.attempt ?? automation.history.length + 1,
    startedAt: timestamp,
    finishedAt:
      update.status === "completed" || update.status === "failed" ? timestamp : undefined,
    notes: update.notes,
    toolResults: update.toolResults.map((result) => ({
      ...result,
      occurredAt: result.occurredAt ?? timestamp,
    })),
  };

  const nextHistory = [run, ...automation.history].slice(0, 25);
  const nextAutomation: GoalAutomationState = {
    ...automation,
    history: nextHistory,
    lastUpdated: timestamp,
    retryAvailable: update.status !== "running",
  };
  card.automation = nextAutomation;

  const activity: ActivityEvent = {
    id: `act-${Date.now()}`,
    type: "automation.triggered",
    actorId: update.agentId,
    actorName: update.agentName,
    boardId,
    description: `${update.agentName} ${update.status} automation for ${card.title}`,
    createdAt: timestamp,
  };

  workspace.activity.unshift(activity);
  workspace.activity = workspace.activity.slice(0, 50);

  await upsertWorkspace(workspace);

  workspaceEventHub.publishAutomation(workspaceId, boardId, cardId, nextAutomation);
  workspaceEventHub.publishActivity(workspaceId, activity);

  return { automation: nextAutomation, activity };
}

export async function createBoard(
  workspaceId: string,
  board: Omit<WorkspaceBoard, "workspaceId" | "metrics">,
  actor: WorkspaceMember
): Promise<{ board: WorkspaceBoard; activity: ActivityEvent }> {
  const workspace = await getWorkspace(workspaceId);
  if (!workspace) {
    throw new Error(`Workspace ${workspaceId} not found`);
  }
  const goalId = board.goalId ?? board.id;
  const newBoard: WorkspaceBoard = {
    ...board,
    workspaceId,
    goalId,
    metrics: await computeBoardMetrics({
      columns: board.columns,
      goalId,
      id: board.id,
    }),
  };
  workspace.boards.push(newBoard);
  const activity: ActivityEvent = {
    id: `act-${Date.now()}`,
    type: "board.created",
    actorId: actor.id,
    actorName: actor.name,
    boardId: newBoard.id,
    description: `${actor.name} created ${newBoard.projectName}`,
    createdAt: new Date().toISOString(),
  };
  workspace.activity.unshift(activity);
  await upsertWorkspace(workspace);
  return { board: newBoard, activity };
}

export async function removeBoard(workspaceId: string, boardId: string, actor: WorkspaceMember): Promise<void> {
  const workspace = await getWorkspace(workspaceId);
  if (!workspace) {
    throw new Error(`Workspace ${workspaceId} not found`);
  }
  workspace.boards = workspace.boards.filter((board) => board.id !== boardId);
  const activity: ActivityEvent = {
    id: `act-${Date.now()}`,
    type: "board.archived",
    actorId: actor.id,
    actorName: actor.name,
    boardId,
    description: `${actor.name} archived board ${boardId}`,
    createdAt: new Date().toISOString(),
  };
  workspace.activity.unshift(activity);
  await upsertWorkspace(workspace);
}

async function computeBoardMetrics(
  board: Pick<WorkspaceBoard, "columns" | "goalId" | "id">
): Promise<BoardMetrics> {
  const completedColumn = board.columns.find((col) => col.title.toLowerCase().includes("done"));
  const completed = completedColumn ? getColumnGoals(completedColumn).length : 0;
  const active = board.columns.reduce((count, column) => count + getColumnGoals(column).length, 0) - completed;
  const vibeMomentum = Math.min(100, Math.max(0, 40 + active * 5 - completed * 3));
  const metrics: BoardMetrics = {
    completedGoals: completed,
    activeGoals: active,
    goalMomentum: vibeMomentum,
  };
  if (board.goalId) {
    const analytics = await findGoalMetric(board.goalId);
    if (analytics) {
      metrics.goalLeadTimeHours = Number((analytics.averageLeadTimeMs / 3_600_000).toFixed(2));
      metrics.goalSuccessRate = Number((analytics.successRate * 100).toFixed(1));
    }
  }
  return metrics;
}
