"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import type {
  ActivityEvent,
  GoalMemoryInsights,
  NotificationEvent,
  PlannerPlan,
  PlannerState,
  PresenceUser,
  UploadReceiptSummary,
  VibeColumn,
  Workspace,
  WorkspaceBoard,
  WorkspaceIntegrationStatus,
  GoalAutomationState,
} from "./board-types";
import type { ClientSessionUser } from "./useSession";
import {
  DEFAULT_CAPABILITY_REGISTRY,
  evaluateFeatureGates,
  type CapabilityFeatureGateStatus,
  type CapabilityRegistry,
  normalizeCapabilityRegistry,
} from "@/shared/capabilities";
import { logError } from "@/shared/logging";
import { isFeatureEnabled } from "./featureFlags";

const accentPalette = [
  "from-indigo-500 via-purple-500 to-blue-500",
  "from-sky-500 via-cyan-400 to-emerald-400",
  "from-rose-500 via-pink-500 to-amber-400",
  "from-violet-500 via-indigo-400 to-fuchsia-500",
  "from-emerald-500 via-lime-400 to-teal-400",
];

type MemorySuggestion = { title: string; detail: string; source: "memory" | "realtime" };

type AssistState = {
  suggestions: { title: string; detail: string }[];
  focusGoal: Goal | null;
  updatedAt: string;
  memory?: GoalMemoryInsights | null;
  longTermSuggestions?: MemorySuggestion[];
} | null;

type AutonomyState = {
  replanTriggered: boolean;
  escalationTriggered: boolean;
  lastTriggeredAt: string | null;
  summary: string | null;
};

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
  uploadReceipts: UploadReceiptSummary[];
  integrations: WorkspaceIntegrationStatus[];
  assist: AssistState;
  autonomy: AutonomyState;
  capabilities: {
    loading: boolean;
    registry: CapabilityRegistry;
    featureGates: CapabilityFeatureGateStatus[];
    has: (token: string) => boolean;
  };
  setWorkspaceId: (workspaceId: string) => void;
  setBoardId: (boardId: string) => void;
  refreshWorkspace: () => Promise<void>;
  refreshBoard: () => Promise<void>;
  createBoard: (projectName: string) => Promise<void>;
  dismissNotification: (id: string) => void;
  requestAssist: () => Promise<void>;
  uploadArtifact: (input: { file: File; dropType: string }) => Promise<void>;
  addColumn: (title: string) => void;
  removeColumn: (columnId: string) => void;
  renameColumn: (columnId: string, title: string) => void;
  addGoal: (columnId: string, title: string, notes?: string) => void;
  updateGoal: (columnId: string, goalId: string, patch: Partial<Goal>) => void;
  removeGoal: (columnId: string, goalId: string) => void;
  moveGoalWithinColumn: (columnId: string, activeId: string, overId: string) => void;
  moveGoalToColumn: (activeColumnId: string, overColumnId: string, activeGoalId: string, overGoalId?: string) => void;
  moveColumn: (activeId: string, overId: string) => void;
  setProjectName: (name: string) => void;
  resumePlan: (token: ResumeToken) => void;
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
  const [uploadReceipts, setUploadReceipts] = useState<UploadReceiptSummary[]>([]);
  const [integrations, setIntegrations] = useState<WorkspaceIntegrationStatus[]>([]);
  const [assist, setAssist] = useState<AssistState>(null);
  const [autonomy, setAutonomy] = useState<AutonomyState>({
    replanTriggered: false,
    escalationTriggered: false,
    lastTriggeredAt: null,
    summary: null,
  });
  const [capabilityRegistry, setCapabilityRegistry] = useState<CapabilityRegistry>(() => ({
    version: DEFAULT_CAPABILITY_REGISTRY.version,
    capabilities: [],
  }));
  const [capabilitiesLoading, setCapabilitiesLoading] = useState(true);
  const [loading, setLoading] = useState(false);
  const [hydrated, setHydrated] = useState(false);

  const eventSourceRef = useRef<EventSource | null>(null);
  const presenceIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const latestBoardRef = useRef<WorkspaceBoard | null>(null);
  const goalEvaluationRef = useRef<{ boardId: string | null; signature: string | null }>({
    boardId: null,
    signature: null,
  });

  const logBoardError = useCallback(
    (event: string, error: unknown, context?: Record<string, unknown>) => {
      logError({
        component: "board.state",
        event,
        message: `Board state failure: ${event}`,
        outcome: "failure",
        error,
        context,
      });
    },
    []
  );

  const ensureBoard = useCallback(() => {
    if (!snapshot) {
      throw new Error("Board snapshot not loaded");
    }
    return snapshot;
  }, [snapshot]);

  const capabilityTokens = useMemo(() => {
    const tokens = new Set<string>();
    for (const capability of capabilityRegistry.capabilities) {
      tokens.add(capability.id);
      for (const provided of capability.provides ?? []) {
        tokens.add(provided);
      }
    }
    return tokens;
  }, [capabilityRegistry]);

  const featureGates = useMemo(
    () => evaluateFeatureGates(capabilityRegistry),
    [capabilityRegistry]
  );

  const hasCapability = useCallback(
    (token: string) => {
      if (capabilitiesLoading) {
        return true;
      }
      return capabilityTokens.has(token);
    },
    [capabilityTokens, capabilitiesLoading]
  );

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
    setNotifications(payload.workspace.notifications ?? []);
    setUploadReceipts(payload.workspace.uploadReceipts ?? []);
    setLoading(false);
    const firstBoard = payload.workspace.boards[0];
    const boardExists = payload.workspace.boards.some((board: WorkspaceBoard) => board.id === boardId);
    if (!boardExists) {
      setBoardId(firstBoard ? firstBoard.id : null);
    }
  }, [workspaceId, boardId]);

  const fetchGoalInsights = useCallback(async () => {
    if (!workspaceId || !boardId) {
      setGoalInsights(null);
      return;
    }
    const response = await fetch(`/api/goals/${boardId}/memory?workspaceId=${workspaceId}`, { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    try {
      const payload = await response.json();
      const normalized = normalizeGoalMemory(payload);
      setGoalInsights(normalized);
      setAssist((previous) =>
        previous
          ? {
              ...previous,
              memory: normalized,
              longTermSuggestions: buildMemorySuggestions(normalized),
            }
          : previous
      );
    } catch (error) {
      logBoardError("goal_memory_parse_failed", error, { workspaceId, boardId });
    }
  }, [workspaceId, boardId, logBoardError]);

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
    void fetchGoalInsights();
  }, [workspaceId, boardId, fetchGoalInsights]);

  const fetchIntegrations = useCallback(async () => {
    if (!workspaceId) return;
    const response = await fetch(`/api/workspaces/${workspaceId}/integrations/status`, { cache: "no-store" });
    if (!response.ok) return;
    const payload = await response.json();
    setIntegrations(payload.integrations ?? []);
  }, [workspaceId]);

  useEffect(() => {
    let active = true;
    setCapabilitiesLoading(true);

    fetch("/api/capabilities")
      .then(async (response) => {
        if (!response.ok) {
          return { version: DEFAULT_CAPABILITY_REGISTRY.version, capabilities: [] };
        }
        try {
          const payload = (await response.json()) as unknown;
          return normalizeCapabilityRegistry(payload);
        } catch {
          return { version: DEFAULT_CAPABILITY_REGISTRY.version, capabilities: [] };
        }
      })
      .then((registry) => {
        if (!active) return;
        setCapabilityRegistry(registry);
      })
      .catch(() => {
        if (!active) return;
        setCapabilityRegistry({
          version: DEFAULT_CAPABILITY_REGISTRY.version,
          capabilities: [],
        });
      })
      .finally(() => {
        if (!active) return;
        setCapabilitiesLoading(false);
      });

    return () => {
      active = false;
    };
  }, []);

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
    fetchWorkspaces().catch((error) => logBoardError("workspaces_fetch_failed", error));
  }, [user, fetchWorkspaces]);

  useEffect(() => {
    if (!workspaceId) return;
    fetchWorkspace().catch((error) =>
      logBoardError("workspace_fetch_failed", error, { workspaceId })
    );
    fetchIntegrations().catch((error) =>
      logBoardError("integrations_fetch_failed", error, { workspaceId })
    );
  }, [workspaceId, fetchWorkspace, fetchIntegrations]);

  useEffect(() => {
    if (!workspaceId || !boardId) return;
    fetchBoard().catch((error) =>
      logBoardError("board_fetch_failed", error, { workspaceId, boardId })
    );
  }, [workspaceId, boardId, fetchBoard]);

  useEffect(() => {
    if (!workspaceId || !boardId) {
      setGoalInsights(null);
      return;
    }
    fetchGoalInsights().catch((error) =>
      logBoardError("goal_memory_fetch_failed", error, { workspaceId, boardId })
    );
  }, [workspaceId, boardId, fetchGoalInsights, logBoardError]);

  useEffect(() => {
    if (!workspaceId || !user) return;
    const eventSource = new EventSource(`/api/workspaces/${workspaceId}/events`);
    eventSource.onmessage = (event) => {
      if (event.type === "message") return;
    };
    eventSource.addEventListener("board-updated", (event) => {
      const payload = JSON.parse(event.data);
      if (payload.boardId === boardId) {
        fetchBoard().catch((error) =>
          logBoardError("board_refresh_failed", error, { workspaceId, boardId })
        );
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
    eventSource.addEventListener("automation", (event) => {
      const payload = JSON.parse(event.data) as {
        boardId: string;
        cardId: string;
        automation: GoalAutomationState;
      };
      if (payload.boardId !== boardId) {
        return;
      }
      setSnapshot((prev) => {
        if (!prev) return prev;
        return {
          ...prev,
          columns: prev.columns.map((column) => ({
            ...column,
            cards: column.cards.map((card) =>
              card.id === payload.cardId ? { ...card, automation: payload.automation } : card
            ),
          })),
        };
      });
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
      }).catch((error) =>
        logBoardError("presence_heartbeat_failed", error, { workspaceId, boardId })
      );
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
      persistBoard(nextBoard).catch((error) =>
        logBoardError("board_persist_failed", error, {
          workspaceId,
          boardId: nextBoard.id,
        })
      );
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
          goals: [],
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

  const createCard = useCallback(
    (partial: Partial<VibeCard> & { title: string }): VibeCard => {
      const id =
        partial.id ?? (typeof crypto !== "undefined" && crypto.randomUUID ? crypto.randomUUID() : `card-${Date.now()}`);
      return {
        id,
        title: partial.title,
        notes: partial.notes ?? "",
        mood: partial.mood ?? "focus",
        createdAt: partial.createdAt ?? new Date().toISOString(),
        assigneeId: partial.assigneeId,
        dueDate: partial.dueDate,
        integrations: partial.integrations ?? [],
        automation: partial.automation ?? {
          goalId: id,
          history: [],
          lastUpdated: new Date().toISOString(),
          retryAvailable: true,
        },
      };
    },
    []
  );

  const addGoal = useCallback(
    (columnId: string, title: string, notes?: string) => {
      updateColumns((columns) =>
        columns.map((column) =>
          column.id === columnId
            ? { ...column, goals: [...column.goals, createGoal({ title, notes })] }
            : column
        )
      );
    },
    [updateColumns, createGoal]
  );

  const updateGoal = useCallback(
    (columnId: string, goalId: string, patch: Partial<Goal>) => {
      updateColumns((columns) =>
        columns.map((column) =>
          column.id === columnId
            ? {
                ...column,
                goals: column.goals.map((goal) =>
                  goal.id === goalId
                    ? {
                        ...goal,
                        ...patch,
                        title: patch.title ? patch.title.trim() || goal.title : goal.title,
                      }
                    : goal
                ),
              }
            : column
        )
      );
    },
    [updateColumns]
  );

  const removeGoal = useCallback(
    (columnId: string, goalId: string) => {
      updateColumns((columns) =>
        columns.map((column) =>
          column.id === columnId
            ? { ...column, goals: column.goals.filter((goal) => goal.id !== goalId) }
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

  const moveGoalWithinColumn = useCallback(
    (columnId: string, activeId: string, overId: string) => {
      updateColumns((columns) =>
        columns.map((column) => {
          if (column.id !== columnId) return column;
          const oldIndex = column.goals.findIndex((goal) => goal.id === activeId);
          const newIndex = column.goals.findIndex((goal) => goal.id === overId);
          if (oldIndex === -1 || newIndex === -1) return column;
          return { ...column, goals: arrayMove(column.goals, oldIndex, newIndex) };
        })
      );
    },
    [updateColumns, arrayMove]
  );

  const moveGoalToColumn = useCallback(
    (activeColumnId: string, overColumnId: string, activeGoalId: string, overGoalId?: string) => {
      if (activeColumnId === overColumnId) return;
      updateColumns((columns) => {
        const sourceIndex = columns.findIndex((column) => column.id === activeColumnId);
        const targetIndex = columns.findIndex((column) => column.id === overColumnId);
        if (sourceIndex === -1 || targetIndex === -1) return columns;
        const sourceColumn = columns[sourceIndex];
        const targetColumn = columns[targetIndex];
        const activeGoalIndex = sourceColumn.goals.findIndex((goal) => goal.id === activeGoalId);
        if (activeGoalIndex === -1) return columns;

        const updatedSourceGoals = [...sourceColumn.goals];
        const [movedGoal] = updatedSourceGoals.splice(activeGoalIndex, 1);

        const updatedTargetGoals = [...targetColumn.goals];
        const insertAt = overGoalId
          ? updatedTargetGoals.findIndex((goal) => goal.id === overGoalId)
          : updatedTargetGoals.length;
        const nextGoals = [...updatedTargetGoals];
        nextGoals.splice(insertAt === -1 ? nextGoals.length : insertAt, 0, movedGoal);

        const nextColumns = [...columns];
        nextColumns[sourceIndex] = { ...sourceColumn, goals: updatedSourceGoals };
        nextColumns[targetIndex] = { ...targetColumn, goals: nextGoals };
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
      persistBoard(nextBoard).catch((error) =>
        logBoardError("board_persist_failed", error, {
          workspaceId,
          boardId: nextBoard.id,
        })
      );
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

  const uploadArtifact = useCallback(
    async ({ file, dropType }: { file: File; dropType: string }) => {
      if (!workspaceId) {
        throw new Error("Workspace required for uploads");
      }
      const formData = new FormData();
      formData.append("workspaceId", workspaceId);
      if (boardId) {
        formData.append("boardId", boardId);
      }
      formData.append("dropType", dropType);
      formData.append("file", file);

      const response = await fetch("/api/uploads", {
        method: "POST",
        body: formData,
      });
      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText || "Upload failed");
      }
      const { notification, upload } = (await response.json()) as {
        notification?: NotificationEvent;
        upload?: UploadReceiptSummary;
      };
      if (notification) {
        setNotifications((prev) => [notification, ...prev].slice(0, 20));
      }
      if (upload) {
        setUploadReceipts((prev) => [upload, ...prev].slice(0, 20));
        setWorkspace((prev) =>
          prev
            ? {
                ...prev,
                uploadReceipts: [upload, ...(prev.uploadReceipts ?? [])].slice(0, 50),
                notifications: notification
                  ? [notification, ...(prev.notifications ?? [])].slice(0, 50)
                  : prev.notifications ?? [],
              }
            : prev
        );
      } else if (notification) {
        setWorkspace((prev) =>
          prev
            ? {
                ...prev,
                notifications: [notification, ...(prev.notifications ?? [])].slice(0, 50),
              }
            : prev
        );
      }
    },
    [workspaceId, boardId]
  );

  useEffect(() => {
    if (!workspaceId || !boardId) return;
    (async () => {
      const response = await fetch(`/api/workspaces/${workspaceId}/boards/${boardId}/assist`, { method: "POST" });
      if (!response.ok) return;
      const payload = await response.json();
      const updatedAt = new Date().toISOString();
      const memory = payload.memory ? normalizeGoalMemory({ ...payload.memory, goalId: `board:${boardId}`, workspaceId }) : goalInsights;
      if (payload.memory && memory) {
        setGoalInsights(memory);
      }
      setAssist({
        suggestions: payload.suggestions ?? [],
        focusCard: payload.focusCard ?? null,
        updatedAt,
        memory: memory ?? null,
        longTermSuggestions: memory ? buildMemorySuggestions(memory) : undefined,
      });
    })();
  }, [workspaceId, boardId, goalInsights]);

  useEffect(() => {
    if (!snapshot || capabilitiesLoading) return;
    if (!snapshot.metrics) return;

    const boardKey = snapshot.id ?? "unknown-board";
    const signature = [
      boardKey,
      snapshot.lastUpdated,
      snapshot.metrics.goalSuccessRate ?? "na",
      snapshot.metrics.goalLeadTimeHours ?? "na",
    ].join("::");

    if (
      goalEvaluationRef.current.boardId === boardKey &&
      goalEvaluationRef.current.signature === signature
    ) {
      return;
    }

    goalEvaluationRef.current = { boardId: boardKey, signature };

    const nowIso = new Date().toISOString();
    const goalInsightsFlag = isFeatureEnabled("goalInsights");
    const autoRetryFlag = isFeatureEnabled("autonomousRetry");
    const escalationFlag = isFeatureEnabled("agentEscalation");
    const hasGoalInsights = goalInsightsFlag && capabilityTokens.has("kanban.goalInsights");
    const autoRetryEnabled = autoRetryFlag && capabilityTokens.has("kanban.autonomousRetry");
    const escalationEnabled = escalationFlag && capabilityTokens.has("kanban.agentEscalation");

    let replanTriggered = false;
    let escalationTriggered = false;
    let summary: string | null = null;

    if (hasGoalInsights && autoRetryEnabled && typeof snapshot.metrics.goalSuccessRate === "number") {
      if (snapshot.metrics.goalSuccessRate < 60) {
        replanTriggered = true;
        summary = `Autonomous retry scheduled at ${snapshot.metrics.goalSuccessRate}% success rate.`;
        requestAssist().catch((error) =>
          logBoardError("autonomous_retry_failed", error, {
            boardId: snapshot.id,
            successRate: snapshot.metrics?.goalSuccessRate,
          })
        );
        setNotifications((prev) => {
          const notification = {
            id: `auto-replan-${Date.now()}`,
            message: `Success rate dipped to ${snapshot.metrics?.goalSuccessRate ?? 0}%. Triggered an autonomous retry cycle.`,
            createdAt: nowIso,
            severity: "warning" as const,
          };
          const items = [notification, ...(prev ?? [])];
          return items.slice(0, 20);
        });
      }
    }

    if (hasGoalInsights && escalationEnabled && typeof snapshot.metrics.goalLeadTimeHours === "number") {
      if (snapshot.metrics.goalLeadTimeHours > 12) {
        escalationTriggered = true;
        const message = `Goal lead time reached ${snapshot.metrics.goalLeadTimeHours}h. Escalating to senior agent.`;
        summary = summary ? `${summary} ${message}` : message;
        setNotifications((prev) => {
          const notification = {
            id: `auto-escalate-${Date.now()}`,
            message,
            createdAt: nowIso,
            severity: "warning" as const,
          };
          const items = [notification, ...(prev ?? [])];
          return items.slice(0, 20);
        });
      }
    }

    if (replanTriggered || escalationTriggered) {
      setAutonomy({
        replanTriggered,
        escalationTriggered,
        lastTriggeredAt: nowIso,
        summary,
      });
    } else {
      setAutonomy((prev) =>
        prev.replanTriggered || prev.escalationTriggered
          ? {
              replanTriggered: false,
              escalationTriggered: false,
              lastTriggeredAt: nowIso,
              summary: null,
            }
          : prev
      );
    }
  }, [
    snapshot,
    capabilitiesLoading,
    capabilityTokens,
    requestAssist,
    logBoardError,
    setNotifications,
  ]);

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
    uploadReceipts,
    integrations,
    assist,
    autonomy,
    capabilities: {
      loading: capabilitiesLoading,
      registry: capabilityRegistry,
      featureGates,
      has: hasCapability,
    },
    setWorkspaceId,
    setBoardId,
    refreshWorkspace,
    refreshBoard,
    createBoard: createBoardMutation,
    dismissNotification,
    requestAssist,
    uploadArtifact,
    retryAutomation,
    addColumn,
    removeColumn,
    renameColumn,
    addGoal,
    updateGoal,
    removeGoal,
    moveGoalWithinColumn,
    moveGoalToColumn,
    moveColumn,
    setProjectName,
    resumePlan,
  };
}

function pickFocusCard(board: WorkspaceBoard | null): VibeCard | null {
  if (!board) return null;
  const inProgress = board.columns.find((column) => column.title.toLowerCase().includes("progress"));
  if (inProgress && inProgress.cards.length) {
    return [...inProgress.cards].sort(
      (a, b) => Date.parse(a.createdAt) - Date.parse(b.createdAt)
    )[0];
  }
  for (const column of board.columns) {
    if (column.cards.length) {
      return column.cards[0];
    }
  }
  return null;
}

function buildGoalSignals(board: WorkspaceBoard): Record<string, unknown>[] {
  const totalCards = board.columns.reduce((count, column) => count + column.cards.length, 0);
  const hypeCards = board.columns.reduce(
    (count, column) => count + column.cards.filter((card) => card.mood === "hype").length,
    0
  );
  const staleCards = board.columns
    .flatMap((column) => column.cards)
    .filter((card) => Date.now() - Date.parse(card.createdAt) > 1000 * 60 * 60 * 24 * 7).length;

  return [
    { name: "totalCards", value: totalCards },
    { name: "hypeCards", value: hypeCards },
    { name: "staleCards", value: staleCards },
  ];
}

type ContinuityEvent = RealTimeEvent & { event_type?: string; workflow_id?: string; payload: Record<string, unknown> };

function getEventType(event: ContinuityEvent): string {
  return event.eventType ?? event.event_type ?? "";
}

function getWorkflowId(event: ContinuityEvent): string {
  return event.workflowId ?? event.workflow_id ?? "";
}

function normalizeWorkflowState(value: unknown): PlannerPlan["status"] | undefined {
  if (typeof value !== "string") return undefined;
  const normalized = value.toLowerCase();
  if (normalized === "pending" || normalized === "running" || normalized === "paused" || normalized === "completed" || normalized === "failed") {
    return normalized;
  }
  return undefined;
}

function normalizeStageState(
  value: unknown
): PlannerPlan["stages"][number]["state"] | undefined {
  if (typeof value !== "string") return undefined;
  const normalized = value.toLowerCase();
  if (normalized === "pending" || normalized === "running" || normalized === "completed" || normalized === "failed" || normalized === "skipped") {
    return normalized;
  }
  return undefined;
}

function normalizeStageIdentifier(value: string): string {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function normalizeResumeToken(value: unknown): ResumeToken | undefined {
  if (!value || typeof value !== "object") return undefined;
  const source = value as Record<string, unknown>;
  const workflowId =
    typeof source.workflowId === "string"
      ? source.workflowId
      : typeof source.workflow_id === "string"
        ? (source.workflow_id as string)
        : undefined;
  if (!workflowId) return undefined;

  const stageId =
    typeof source.stageId === "string"
      ? (source.stageId as string)
      : typeof source.stage_id === "string"
        ? (source.stage_id as string)
        : undefined;

  const checkpoint =
    typeof source.checkpoint === "string" ? (source.checkpoint as string) : "";
  const issuedAt =
    typeof source.issuedAt === "string"
      ? (source.issuedAt as string)
      : typeof source.issued_at === "string"
        ? (source.issued_at as string)
        : new Date().toISOString();
  const expiresAt =
    typeof source.expiresAt === "string"
      ? (source.expiresAt as string)
      : typeof source.expires_at === "string"
        ? (source.expires_at as string)
        : new Date().toISOString();

  return {
    workflowId,
    stageId,
    checkpoint,
    issuedAt,
    expiresAt,
  };
}

function mergeStageStates(
  stages: PlannerPlan["stages"],
  stageId: string,
  state: PlannerPlan["stages"][number]["state"]
): PlannerPlan["stages"] {
  const existing = stages.find((stage) => stage.id === stageId);
  if (!existing) {
    return [...stages, { id: stageId, name: stageId, state }];
  }
  return stages.map((stage) => (stage.id === stageId ? { ...stage, state } : stage));
}

function reducePlannerFromEvent(prev: PlannerState, event: RealTimeEvent): PlannerState {
  const workflowId = getWorkflowId(event as ContinuityEvent);
  if (!workflowId) {
    return prev;
  }

  const timestamp = typeof event.timestamp === "string" ? event.timestamp : new Date().toISOString();
  let handled = false;

  const plans = prev.plans.map((plan) => {
    if (plan.workflowId !== workflowId) {
      return plan;
    }

    handled = true;
    const eventType = getEventType(event as ContinuityEvent);
    const payload = event.payload ?? {};
    let nextPlan: PlannerPlan = { ...plan, updatedAt: timestamp };

    if (eventType === "workflow/state") {
      const nextState = normalizeWorkflowState(payload.state);
      if (nextState) {
        nextPlan = { ...nextPlan, status: nextState };
        if (nextState === "completed" || nextState === "failed") {
          nextPlan = { ...nextPlan, resumeToken: undefined };
        }
      }
      const resumeToken = normalizeResumeToken(payload.resumeToken ?? payload.resume_token);
      if (resumeToken) {
        nextPlan = { ...nextPlan, resumeToken };
      }
    } else if (eventType === "workflow/stage") {
      const stageIdentifier = payload.stage_id ?? payload.stageId;
      const stageState = normalizeStageState(payload.state);
      if (typeof stageIdentifier === "string" && stageState) {
        const stageId = normalizeStageIdentifier(stageIdentifier);
        nextPlan = {
          ...nextPlan,
          stages: mergeStageStates(nextPlan.stages, stageId, stageState),
        };
        if (stageState === "running" && nextPlan.status === "pending") {
          nextPlan = { ...nextPlan, status: "running" };
        }
        if (stageState === "failed") {
          nextPlan = { ...nextPlan, status: "failed" };
        }
      }
      const resumeToken = normalizeResumeToken(payload.resumeToken ?? payload.resume_token);
      if (resumeToken) {
        nextPlan = { ...nextPlan, resumeToken };
      }
    } else if (eventType === "workflow/resume") {
      const resumeToken = normalizeResumeToken(payload.resumeToken ?? payload.token);
      if (resumeToken) {
        nextPlan = { ...nextPlan, resumeToken };
      }
    }

    return nextPlan;
  });

  if (!handled) {
    return prev;
  }

  let plannerStatus = prev.status === "idle" ? "ready" : prev.status;
  let activePlanId = prev.activePlanId;
  const updatedPlan = plans.find((plan) => plan.workflowId === workflowId);

  if (updatedPlan) {
    if (updatedPlan.status === "completed" || updatedPlan.status === "failed") {
      if (prev.activePlanId === updatedPlan.goalId) {
        activePlanId = plans.find(
          (plan) =>
            plan.workflowId !== workflowId &&
            (plan.status === "running" || plan.status === "pending" || plan.status === "paused")
        )?.goalId;
      }
      if (updatedPlan.status === "failed") {
        plannerStatus = "error";
      }
    } else {
      activePlanId = updatedPlan.goalId;
      if (updatedPlan.status === "running") {
        plannerStatus = plannerStatus === "planning" ? "planning" : "ready";
      }
    }
  }

  return {
    status: plannerStatus,
    plans,
    activePlanId,
    lastError: prev.lastError,
  };
}

function normalizeGoalMemory(payload: unknown): GoalMemoryInsights {
  const value = (typeof payload === "object" && payload) || {};
  const summary = typeof (value as { summary?: unknown }).summary === "string" ? (value as { summary: string }).summary : "";
  const traceCountRaw = (value as { traceCount?: unknown }).traceCount;
  const tracesRaw = (value as { traces?: unknown }).traces;
  const traceCount = Number.isFinite(traceCountRaw)
    ? Number(traceCountRaw)
    : Array.isArray(tracesRaw)
      ? tracesRaw.length
      : 0;
  const lastSeenRaw = (value as { lastSeen?: unknown }).lastSeen;
  const lastSeen = typeof lastSeenRaw === "string" ? lastSeenRaw : null;
  const traces = Array.isArray(tracesRaw)
    ? tracesRaw.map((trace) => ({
        id: String((trace as { id?: unknown }).id ?? cryptoRandom()),
        goalId: String((trace as { goalId?: unknown }).goalId ?? ""),
        workspaceId: String((trace as { workspaceId?: unknown }).workspaceId ?? ""),
        boardId: (trace as { boardId?: unknown }).boardId ?? null,
        actorId: (trace as { actorId?: unknown }).actorId ? String((trace as { actorId?: unknown }).actorId) : null,
        actorName: (trace as { actorName?: unknown }).actorName ? String((trace as { actorName?: unknown }).actorName) : null,
        action: String((trace as { action?: unknown }).action ?? "unknown"),
        summary: (trace as { summary?: unknown }).summary ? String((trace as { summary?: unknown }).summary) : null,
        metadata:
          (trace as { metadata?: unknown }).metadata && typeof (trace as { metadata?: unknown }).metadata === "object"
            ? ((trace as { metadata: Record<string, unknown> }).metadata as Record<string, unknown>)
            : null,
        createdAt: String((trace as { createdAt?: unknown }).createdAt ?? new Date().toISOString()),
      }))
    : [];
  const lifecycleRaw = (value as { lifecycle?: unknown }).lifecycle;
  const lifecycle = Array.isArray(lifecycleRaw)
    ? lifecycleRaw.map((event) => ({
        id: Number((event as { id?: unknown }).id ?? Date.now()),
        goalId: String((event as { goalId?: unknown }).goalId ?? ""),
        workspaceId: String((event as { workspaceId?: unknown }).workspaceId ?? ""),
        eventType: String((event as { eventType?: unknown }).eventType ?? "unknown"),
        status: (event as { status?: unknown }).status ? String((event as { status?: unknown }).status) : null,
        summary: (event as { summary?: unknown }).summary ? String((event as { summary?: unknown }).summary) : null,
        payload: (event as { payload?: unknown }).payload,
        createdAt: String((event as { createdAt?: unknown }).createdAt ?? new Date().toISOString()),
      }))
    : [];
  const artifactsRaw = (value as { artifacts?: unknown }).artifacts;
  const artifacts = Array.isArray(artifactsRaw)
    ? artifactsRaw.map((artifact) => ({
        id: Number((artifact as { id?: unknown }).id ?? Date.now()),
        goalId: String((artifact as { goalId?: unknown }).goalId ?? ""),
        workspaceId: String((artifact as { workspaceId?: unknown }).workspaceId ?? ""),
        artifactType: String((artifact as { artifactType?: unknown }).artifactType ?? "unknown"),
        artifactUri: String((artifact as { artifactUri?: unknown }).artifactUri ?? ""),
        title: (artifact as { title?: unknown }).title ? String((artifact as { title?: unknown }).title) : null,
        summary: (artifact as { summary?: unknown }).summary ? String((artifact as { summary?: unknown }).summary) : null,
        metadata:
          (artifact as { metadata?: unknown }).metadata && typeof (artifact as { metadata?: unknown }).metadata === "object"
            ? ((artifact as { metadata: Record<string, unknown> }).metadata as Record<string, unknown>)
            : null,
        createdAt: String((artifact as { createdAt?: unknown }).createdAt ?? new Date().toISOString()),
      }))
    : [];
  const similarGoalsRaw = (value as { similarGoals?: unknown }).similarGoals;
  const similarGoals = Array.isArray(similarGoalsRaw)
    ? similarGoalsRaw.map((goal) => ({
        goalId: String((goal as { goalId?: unknown }).goalId ?? ""),
        workspaceId: String((goal as { workspaceId?: unknown }).workspaceId ?? ""),
        score: Number((goal as { score?: unknown }).score ?? 0),
      }))
    : [];
  const insightSummaryRaw = (value as { insightSummary?: unknown }).insightSummary;
  const insightSummary = typeof insightSummaryRaw === "string" ? insightSummaryRaw : undefined;
  return {
    summary,
    traceCount,
    lastSeen,
    traces,
    lifecycle,
    artifacts,
    similarGoals,
    insightSummary,
    updatedAt: new Date().toISOString(),
  };
}

function buildMemorySuggestions(memory: GoalMemoryInsights): MemorySuggestion[] {
  const suggestions: MemorySuggestion[] = [];
  if (memory.traceCount > 0) {
    suggestions.push({
      title: "Replay last successful move",
      detail: `Last execution recorded ${memory.lastSeen ? new Date(memory.lastSeen).toLocaleString() : "recently"}. Revisit the stored steps before planning next actions.`,
      source: "memory",
    });
  }
  if (memory.artifacts.length) {
    const latestArtifact = memory.artifacts[0];
    suggestions.push({
      title: "Review stored artifact",
      detail: `Artifact ${latestArtifact.title ?? latestArtifact.artifactType} is available at ${latestArtifact.artifactUri}.`,
      source: "memory",
    });
  }
  if (memory.similarGoals.length) {
    const topMatch = memory.similarGoals[0];
    suggestions.push({
      title: "Borrow from similar goal",
      detail: `Goal ${topMatch.goalId} (${topMatch.score.toFixed(2)}) in workspace ${topMatch.workspaceId} may contain reusable playbooks.`,
      source: "memory",
    });
  }
  if (!suggestions.length && memory.summary) {
    suggestions.push({
      title: "No memory backlog",
      detail: memory.summary,
      source: "memory",
    });
  }
  return suggestions;
}

function cryptoRandom() {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  return `${Date.now()}-${Math.random().toString(16).slice(2, 10)}`;
}
