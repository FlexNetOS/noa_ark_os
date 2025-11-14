import { promises as fs } from "fs";
import path from "path";

import { recordWorkspaceSnapshot } from "./memory-store";

import type {
  ActivityEvent,
  NotificationEvent,
  UploadReceiptSummary,
  Workspace,
  WorkspaceBoard,
  WorkspaceMember,
} from "../app/components/board-types";

const DATA_DIR = path.join(process.cwd(), "data");
const DATA_FILE = path.join(DATA_DIR, "workspaces.json");

type WorkspaceStore = {
  workspaces: Workspace[];
};

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
          projectName: "Vibe Coding Launchpad",
          description: "Ship the new kanban workspace with presence-aware collaboration.",
          lastUpdated: now,
          accent: "from-indigo-500 via-purple-500 to-blue-500",
          columns: [
            {
              id: "todo",
              title: "To Do",
              accent: "from-indigo-500 via-purple-500 to-blue-500",
              cards: [
                {
                  id: "card-1",
                  title: "Ideate hero motion",
                  notes: "Sketch the flowing hero animation that loops smoothly across the dashboard.",
                  createdAt: now,
                  mood: "flow"
                },
                {
                  id: "card-2",
                  title: "Sound-reactive palette",
                  notes: "Research how to tie track BPM to gradient shifts on the home screen.",
                  createdAt: now,
                  mood: "focus"
                }
              ]
            },
            {
              id: "in-progress",
              title: "In Progress",
              accent: "from-sky-500 via-cyan-400 to-emerald-400",
              cards: [
                {
                  id: "card-3",
                  title: "Prototype kanban drag",
                  notes: "Polish easing curve + inertia for drag transitions.",
                  createdAt: now,
                  mood: "hype"
                }
              ]
            },
            {
              id: "done",
              title: "Completed",
              accent: "from-violet-500 via-indigo-400 to-fuchsia-500",
              cards: [
                {
                  id: "card-4",
                  title: "Mood-driven theme",
                  notes: "Shipped gradient system that syncs with vibes.",
                  createdAt: now,
                  mood: "chill"
                }
              ]
            }
          ],
          metrics: {
            completedCards: 1,
            activeCards: 3,
            vibeMomentum: 72
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
  return {
    workspaces: parsed.workspaces.map((workspace) => ({
      ...workspace,
      notifications: workspace.notifications ?? [],
      uploadReceipts: workspace.uploadReceipts ?? [],
    })),
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
  if (boardIndex === -1) {
    workspace.boards.push(nextBoard);
  } else {
    workspace.boards[boardIndex] = nextBoard;
  }

  const activity: ActivityEvent = {
    id: `act-${Date.now()}`,
    type: "board.updated",
    actorId: actor.id,
    actorName: actor.name,
    boardId: nextBoard.id,
    description: `${actor.name} synced ${nextBoard.projectName}`,
    createdAt: new Date().toISOString(),
  };
  workspace.activity.unshift(activity);
  workspace.activity = workspace.activity.slice(0, 50);

  nextBoard.metrics = computeBoardMetrics(nextBoard);

  await upsertWorkspace({ ...workspace, lastSyncedAt: new Date().toISOString() } as Workspace);
  return { board: nextBoard, activity };
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

export async function createBoard(
  workspaceId: string,
  board: Omit<WorkspaceBoard, "workspaceId" | "metrics">,
  actor: WorkspaceMember
): Promise<{ board: WorkspaceBoard; activity: ActivityEvent }> {
  const workspace = await getWorkspace(workspaceId);
  if (!workspace) {
    throw new Error(`Workspace ${workspaceId} not found`);
  }
  const newBoard: WorkspaceBoard = {
    ...board,
    workspaceId,
    metrics: computeBoardMetrics(board),
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

function computeBoardMetrics(board: Pick<WorkspaceBoard, "columns">): WorkspaceBoard["metrics"] {
  const completed = board.columns.find((col) => col.title.toLowerCase().includes("done"))?.cards.length ?? 0;
  const active = board.columns.reduce((count, column) => count + column.cards.length, 0) - completed;
  const vibeMomentum = Math.min(100, Math.max(0, 40 + active * 5 - completed * 3));
  return {
    completedCards: completed,
    activeCards: active,
    vibeMomentum,
  };
}
