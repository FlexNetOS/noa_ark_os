export type VibeMood = "focus" | "flow" | "chill" | "hype";

export type CardIntegrationSnapshot = {
  kind: "crc" | "cicd" | "runtime" | "agent";
  label: string;
  status: "idle" | "running" | "success" | "failed";
  details?: string;
};

export type VibeCard = {
  id: string;
  title: string;
  notes: string;
  createdAt: string;
  mood: VibeMood;
  assigneeId?: string;
  dueDate?: string;
  integrations?: CardIntegrationSnapshot[];
};

export type VibeColumn = {
  id: string;
  title: string;
  accent: string;
  cards: VibeCard[];
};

export type BoardMoodSample = {
  recordedAt: string;
  focus: number;
  flow: number;
  chill: number;
  hype: number;
};

export type BoardMetrics = {
  completedCards: number;
  activeCards: number;
  vibeMomentum: number;
  cycleTimeDays?: number;
  flowEfficiency?: number;
};

export type BoardSnapshot = {
  id?: string;
  workspaceId?: string;
  columns: VibeColumn[];
  lastUpdated: string;
  projectName: string;
  description?: string;
  accent?: string;
  archived?: boolean;
  metrics?: BoardMetrics;
  moodSamples?: BoardMoodSample[];
};

export type WorkspaceBoard = BoardSnapshot & {
  id: string;
  workspaceId: string;
};

export type WorkspaceMember = {
  id: string;
  name: string;
  role: "owner" | "member";
  avatarHue: number;
};

export type ActivityEvent = {
  id: string;
  type: "board.created" | "board.updated" | "board.archived" | "presence.joined" | "presence.left" | "automation.triggered";
  actorId: string;
  actorName: string;
  boardId?: string;
  description: string;
  createdAt: string;
};

export type Workspace = {
  id: string;
  name: string;
  accent: string;
  createdAt: string;
  billingPlan: "starter" | "growth" | "enterprise";
  members: WorkspaceMember[];
  boards: WorkspaceBoard[];
  activity: ActivityEvent[];
  notifications: NotificationEvent[];
  uploadReceipts: UploadReceiptSummary[];
  lastSyncedAt?: string;
};

export type PresenceUser = {
  userId: string;
  userName: string;
  status: "online" | "idle";
  lastPing: string;
};

export type WorkspacePresenceState = {
  workspaceId: string;
  boardId?: string;
  users: PresenceUser[];
};

export type NotificationEvent = {
  id: string;
  message: string;
  createdAt: string;
  severity: "info" | "success" | "warning" | "error";
  href?: string;
  casKeys?: string[];
  receiptPath?: string;
};

export type WorkspaceIntegrationStatus = {
  id: string;
  name: string;
  status: "healthy" | "running" | "degraded" | "error";
  lastEvent: string;
};

export type UploadReceiptSummary = {
  id: string;
  workspaceId: string;
  boardId?: string;
  dropId: string;
  dropType: string;
  originalName: string;
  casKeys: string[];
  receiptPath: string;
  uploadedAt: string;
  uploadedBy: {
    id: string;
    name: string;
  };
};

export type GoalMemoryTrace = {
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

export type GoalLifecycleEventSnapshot = {
  id: number;
  goalId: string;
  workspaceId: string;
  eventType: string;
  status?: string | null;
  summary?: string | null;
  payload?: unknown;
  createdAt: string;
};

export type GoalArtifactSnapshot = {
  id: number;
  goalId: string;
  workspaceId: string;
  artifactType: string;
  artifactUri: string;
  title?: string | null;
  summary?: string | null;
  metadata?: Record<string, unknown> | null;
  createdAt: string;
};

export type GoalMemorySimilarGoal = {
  goalId: string;
  workspaceId: string;
  score: number;
};

export type GoalMemoryInsights = {
  summary: string;
  traceCount: number;
  lastSeen: string | null;
  traces: GoalMemoryTrace[];
  lifecycle: GoalLifecycleEventSnapshot[];
  artifacts: GoalArtifactSnapshot[];
  similarGoals: GoalMemorySimilarGoal[];
  insightSummary?: string;
  updatedAt: string;
};
