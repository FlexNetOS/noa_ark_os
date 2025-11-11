"use client";

import { useCallback, useEffect, useRef, useState } from "react";

import type {
  ActivityEvent,
  NotificationEvent,
  PresenceUser,
  VibeCard,
  VibeColumn,
  Workspace,
  WorkspaceBoard,
  WorkspaceIntegrationStatus,
} from "./board-types";
import type { ClientSessionUser } from "./useSession";

const accentPalette = [
  "from-indigo-500 via-purple-500 to-blue-500",
  "from-sky-500 via-cyan-400 to-emerald-400",
  "from-rose-500 via-pink-500 to-amber-400",
  "from-violet-500 via-indigo-400 to-fuchsia-500",
  "from-emerald-500 via-lime-400 to-teal-400",
];

type AssistState = {
  suggestions: { title: string; detail: string }[];
  focusCard: VibeCard | null;
  updatedAt: string;
} | null;

type WorkspaceHookState = {
  hydrated: boolean;
  loading: boolean;
  user: ClientSessionUser | null;
  workspaceId: string | null;
  boardId: string | null;
  workspaces: Workspace[];
  workspace: Workspace | null;
  snapshot: WorkspaceBoard | null;
  presence: PresenceUser[];
  activity: ActivityEvent[];
  notifications: NotificationEvent[];
  integrations: WorkspaceIntegrationStatus[];
  assist: AssistState;
  setWorkspaceId: (workspaceId: string) => void;
  setBoardId: (boardId: string) => void;
  refreshWorkspace: () => Promise<void>;
  refreshBoard: () => Promise<void>;
  createBoard: (projectName: string) => Promise<void>;
  dismissNotification: (id: string) => void;
  requestAssist: () => Promise<void>;
  addColumn: (title: string) => void;
  removeColumn: (columnId: string) => void;
  renameColumn: (columnId: string, title: string) => void;
  addCard: (columnId: string, title: string, notes?: string) => void;
  updateCard: (columnId: string, cardId: string, patch: Partial<VibeCard>) => void;
  removeCard: (columnId: string, cardId: string) => void;
  moveCardWithinColumn: (columnId: string, activeId: string, overId: string) => void;
  moveCardToColumn: (activeColumnId: string, overColumnId: string, activeCardId: string, overCardId?: string) => void;
  moveColumn: (activeId: string, overId: string) => void;
  setProjectName: (name: string) => void;
};

export type BoardState = WorkspaceHookState;

export function useBoardState(user: ClientSessionUser | null): WorkspaceHookState {
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [workspaceId, setWorkspaceId] = useState<string | null>(null);
  const [workspace, setWorkspace] = useState<Workspace | null>(null);
  const [boardId, setBoardId] = useState<string | null>(null);
  const [snapshot, setSnapshot] = useState<WorkspaceBoard | null>(null);
  const [presence, setPresence] = useState<PresenceUser[]>([]);
  const [activity, setActivity] = useState<ActivityEvent[]>([]);
  const [notifications, setNotifications] = useState<NotificationEvent[]>([]);
  const [integrations, setIntegrations] = useState<WorkspaceIntegrationStatus[]>([]);
  const [assist, setAssist] = useState<AssistState>(null);
  const [loading, setLoading] = useState(false);
  const [hydrated, setHydrated] = useState(false);

  const eventSourceRef = useRef<EventSource | null>(null);
  const presenceIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const latestBoardRef = useRef<WorkspaceBoard | null>(null);

  const ensureBoard = useCallback(() => {
    if (!snapshot) {
      throw new Error("Board snapshot not loaded");
    }
    return snapshot;
  }, [snapshot]);

  const fetchWorkspaces = useCallback(async () => {
    if (!user) return;
    setLoading(true);
    const response = await fetch("/api/workspaces", { cache: "no-store" });
    if (!response.ok) {
      throw new Error("Failed to load workspaces");
    }
    const payload = await response.json();
    setWorkspaces(payload.workspaces ?? []);
    setLoading(false);
    if (!workspaceId && payload.workspaces?.length) {
      setWorkspaceId(payload.workspaces[0].id);
    }
  }, [user, workspaceId]);

  const fetchWorkspace = useCallback(async () => {
    if (!workspaceId) return;
    setLoading(true);
    const response = await fetch(`/api/workspaces/${workspaceId}`, { cache: "no-store" });
    if (!response.ok) {
      throw new Error("Failed to load workspace");
    }
    const payload = await response.json();
    setWorkspace(payload.workspace);
    setActivity(payload.workspace.activity ?? []);
    setLoading(false);
    const firstBoard = payload.workspace.boards[0];
    const boardExists = payload.workspace.boards.some((board: WorkspaceBoard) => board.id === boardId);
    if (!boardExists) {
      setBoardId(firstBoard ? firstBoard.id : null);
    }
  }, [workspaceId, boardId]);

  const fetchBoard = useCallback(async () => {
    if (!workspaceId || !boardId) return;
    const response = await fetch(`/api/workspaces/${workspaceId}/boards/${boardId}`, { cache: "no-store" });
    if (!response.ok) {
      throw new Error("Failed to load board");
    }
    const payload = await response.json();
    setSnapshot(payload.board);
    latestBoardRef.current = payload.board;
    setHydrated(true);
  }, [workspaceId, boardId]);

  const fetchIntegrations = useCallback(async () => {
    if (!workspaceId) return;
    const response = await fetch(`/api/workspaces/${workspaceId}/integrations/status`, { cache: "no-store" });
    if (!response.ok) return;
    const payload = await response.json();
    setIntegrations(payload.integrations ?? []);
  }, [workspaceId]);

  useEffect(() => {
    if (!user) {
      setWorkspaces([]);
      setWorkspaceId(null);
      setWorkspace(null);
      setBoardId(null);
      setSnapshot(null);
      setHydrated(false);
      return;
    }
    fetchWorkspaces().catch((error) => console.error(error));
  }, [user, fetchWorkspaces]);

  useEffect(() => {
    if (!workspaceId) return;
    fetchWorkspace().catch((error) => console.error(error));
    fetchIntegrations().catch((error) => console.error(error));
  }, [workspaceId, fetchWorkspace, fetchIntegrations]);

  useEffect(() => {
    if (!workspaceId || !boardId) return;
    fetchBoard().catch((error) => console.error(error));
  }, [workspaceId, boardId, fetchBoard]);

  useEffect(() => {
    if (!workspaceId || !user) return;
    const eventSource = new EventSource(`/api/workspaces/${workspaceId}/events`);
    eventSource.onmessage = (event) => {
      if (event.type === "message") return;
    };
    eventSource.addEventListener("board-updated", (event) => {
      const payload = JSON.parse(event.data);
      if (payload.boardId === boardId) {
        fetchBoard().catch((error) => console.error(error));
      }
    });
    eventSource.addEventListener("activity", (event) => {
      const payload = JSON.parse(event.data) as ActivityEvent;
      setActivity((prev) => [payload, ...prev].slice(0, 50));
    });
    eventSource.addEventListener("notification", (event) => {
      const payload = JSON.parse(event.data) as NotificationEvent;
      setNotifications((prev) => [payload, ...prev].slice(0, 20));
    });
    eventSource.addEventListener("presence", (event) => {
      const payload = JSON.parse(event.data) as { users: PresenceUser[] };
      setPresence(payload.users ?? []);
    });
    eventSourceRef.current = eventSource;
    return () => {
      eventSource.close();
      eventSourceRef.current = null;
    };
  }, [workspaceId, boardId, user, fetchBoard]);

  useEffect(() => {
    if (!workspaceId || !user) return;
    const sendHeartbeat = () => {
      fetch(`/api/workspaces/${workspaceId}/presence`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ boardId }),
      }).catch((error) => console.error(error));
    };
    sendHeartbeat();
    const interval = setInterval(sendHeartbeat, 12_000);
    presenceIntervalRef.current = interval;
    return () => {
      clearInterval(interval);
      presenceIntervalRef.current = null;
      fetch(`/api/workspaces/${workspaceId}/presence`, { method: "DELETE" }).catch(() => undefined);
    };
  }, [workspaceId, boardId, user]);

  const persistBoard = useCallback(
    async (nextBoard: WorkspaceBoard) => {
      if (!workspaceId || !nextBoard.id) return;
      latestBoardRef.current = nextBoard;
      setSnapshot(nextBoard);
      await fetch(`/api/workspaces/${workspaceId}/boards/${nextBoard.id}`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(nextBoard),
      });
    },
    [workspaceId]
  );

  const updateColumns = useCallback(
    (updater: (columns: VibeColumn[]) => VibeColumn[]) => {
      const board = ensureBoard();
      const nextBoard: WorkspaceBoard = {
        ...board,
        columns: updater(board.columns),
      };
      persistBoard(nextBoard).catch((error) => console.error(error));
    },
    [ensureBoard, persistBoard]
  );

  const addColumn = useCallback(
    (title: string) => {
      updateColumns((columns) => [
        ...columns,
        {
          id: `column-${Date.now()}`,
          title: title.trim() || "Untitled",
          accent: accentPalette[columns.length % accentPalette.length],
          cards: [],
        },
      ]);
    },
    [updateColumns]
  );

  const removeColumn = useCallback(
    (columnId: string) => {
      updateColumns((columns) => columns.filter((column) => column.id !== columnId));
    },
    [updateColumns]
  );

  const renameColumn = useCallback(
    (columnId: string, title: string) => {
      updateColumns((columns) =>
        columns.map((column) => (column.id === columnId ? { ...column, title: title.trim() || column.title } : column))
      );
    },
    [updateColumns]
  );

  const createCard = useCallback((partial: Partial<VibeCard> & { title: string }): VibeCard => {
    const id = partial.id ?? (typeof crypto !== "undefined" && crypto.randomUUID ? crypto.randomUUID() : `card-${Date.now()}`);
    return {
      id,
      title: partial.title,
      notes: partial.notes ?? "",
      mood: partial.mood ?? "focus",
      createdAt: partial.createdAt ?? new Date().toISOString(),
      assigneeId: partial.assigneeId,
      dueDate: partial.dueDate,
      integrations: partial.integrations ?? [],
    };
  }, []);

  const addCard = useCallback(
    (columnId: string, title: string, notes?: string) => {
      updateColumns((columns) =>
        columns.map((column) =>
          column.id === columnId
            ? { ...column, cards: [...column.cards, createCard({ title, notes })] }
            : column
        )
      );
    },
    [updateColumns, createCard]
  );

  const updateCard = useCallback(
    (columnId: string, cardId: string, patch: Partial<VibeCard>) => {
      updateColumns((columns) =>
        columns.map((column) =>
          column.id === columnId
            ? {
                ...column,
                cards: column.cards.map((card) =>
                  card.id === cardId
                    ? {
                        ...card,
                        ...patch,
                        title: patch.title ? patch.title.trim() || card.title : card.title,
                      }
                    : card
                ),
              }
            : column
        )
      );
    },
    [updateColumns]
  );

  const removeCard = useCallback(
    (columnId: string, cardId: string) => {
      updateColumns((columns) =>
        columns.map((column) =>
          column.id === columnId
            ? { ...column, cards: column.cards.filter((card) => card.id !== cardId) }
            : column
        )
      );
    },
    [updateColumns]
  );

  const arrayMove = useCallback(<T,>(list: T[], from: number, to: number) => {
    if (from === to) return list;
    const next = [...list];
    const [item] = next.splice(from, 1);
    next.splice(to, 0, item);
    return next;
  }, []);

  const moveCardWithinColumn = useCallback(
    (columnId: string, activeId: string, overId: string) => {
      updateColumns((columns) =>
        columns.map((column) => {
          if (column.id !== columnId) return column;
          const oldIndex = column.cards.findIndex((card) => card.id === activeId);
          const newIndex = column.cards.findIndex((card) => card.id === overId);
          if (oldIndex === -1 || newIndex === -1) return column;
          return { ...column, cards: arrayMove(column.cards, oldIndex, newIndex) };
        })
      );
    },
    [updateColumns, arrayMove]
  );

  const moveCardToColumn = useCallback(
    (activeColumnId: string, overColumnId: string, activeCardId: string, overCardId?: string) => {
      if (activeColumnId === overColumnId) return;
      updateColumns((columns) => {
        const sourceIndex = columns.findIndex((column) => column.id === activeColumnId);
        const targetIndex = columns.findIndex((column) => column.id === overColumnId);
        if (sourceIndex === -1 || targetIndex === -1) return columns;
        const sourceColumn = columns[sourceIndex];
        const targetColumn = columns[targetIndex];
        const activeCardIndex = sourceColumn.cards.findIndex((card) => card.id === activeCardId);
        if (activeCardIndex === -1) return columns;

        const updatedSourceCards = [...sourceColumn.cards];
        const [movedCard] = updatedSourceCards.splice(activeCardIndex, 1);

        const updatedTargetCards = [...targetColumn.cards];
        const insertAt = overCardId ? updatedTargetCards.findIndex((card) => card.id === overCardId) : updatedTargetCards.length;
        const nextCards = [...updatedTargetCards];
        nextCards.splice(insertAt === -1 ? nextCards.length : insertAt, 0, movedCard);

        const nextColumns = [...columns];
        nextColumns[sourceIndex] = { ...sourceColumn, cards: updatedSourceCards };
        nextColumns[targetIndex] = { ...targetColumn, cards: nextCards };
        return nextColumns;
      });
    },
    [updateColumns]
  );

  const moveColumn = useCallback(
    (activeId: string, overId: string) => {
      if (activeId === overId) return;
      updateColumns((columns) => {
        const oldIndex = columns.findIndex((column) => column.id === activeId);
        const newIndex = columns.findIndex((column) => column.id === overId);
        if (oldIndex === -1 || newIndex === -1) return columns;
        return arrayMove(columns, oldIndex, newIndex);
      });
    },
    [updateColumns, arrayMove]
  );

  const setProjectName = useCallback(
    (name: string) => {
      const board = ensureBoard();
      const nextBoard: WorkspaceBoard = {
        ...board,
        projectName: name.trim() || board.projectName,
      };
      persistBoard(nextBoard).catch((error) => console.error(error));
    },
    [ensureBoard, persistBoard]
  );

  const createBoardMutation = useCallback(
    async (projectName: string) => {
      if (!workspaceId) return;
      const response = await fetch(`/api/workspaces/${workspaceId}/boards`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ projectName }),
      });
      if (!response.ok) {
        throw new Error("Failed to create board");
      }
      const payload = await response.json();
      setWorkspace((current) => {
        if (!current) return current;
        return { ...current, boards: [payload.board, ...current.boards] };
      });
      setBoardId(payload.board.id);
    },
    [workspaceId]
  );

  const dismissNotification = useCallback((id: string) => {
    setNotifications((prev) => prev.filter((notification) => notification.id !== id));
  }, []);

  const requestAssist = useCallback(async () => {
    if (!workspaceId || !boardId) return;
    const response = await fetch(`/api/workspaces/${workspaceId}/boards/${boardId}/assist`, { method: "POST" });
    if (!response.ok) return;
    const payload = await response.json();
    setAssist({ suggestions: payload.suggestions ?? [], focusCard: payload.focusCard ?? null, updatedAt: new Date().toISOString() });
  }, [workspaceId, boardId]);

  const refreshWorkspace = useCallback(async () => {
    await fetchWorkspace();
  }, [fetchWorkspace]);

  const refreshBoard = useCallback(async () => {
    await fetchBoard();
  }, [fetchBoard]);

  return {
    hydrated,
    loading,
    user,
    workspaceId,
    boardId,
    workspaces,
    workspace,
    snapshot,
    presence,
    activity,
    notifications,
    integrations,
    assist,
    setWorkspaceId,
    setBoardId,
    refreshWorkspace,
    refreshBoard,
    createBoard: createBoardMutation,
    dismissNotification,
    requestAssist,
    addColumn,
    removeColumn,
    renameColumn,
    addCard,
    updateCard,
    removeCard,
    moveCardWithinColumn,
    moveCardToColumn,
    moveColumn,
    setProjectName,
  };
}
