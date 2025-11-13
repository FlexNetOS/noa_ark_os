export type VibeMood = "focus" | "flow" | "chill" | "hype";

export type CardIntegrationSnapshot = {
  kind: "crc" | "cicd" | "runtime" | "agent";
  label: string;
  status: "idle" | "running" | "success" | "failed";
  details?: string;
};

export type ToolExecutionTelemetry = {
  name: string;
  capability: string;
  status: "pending" | "running" | "succeeded" | "failed" | "skipped";
  output?: string;
  error?: string;
  occurredAt?: string;
};

export type AgentAutomationRun = {
  agentId: string;
  agentName: string;
  status: "queued" | "running" | "completed" | "failed";
  attempt: number;
  startedAt: string;
  finishedAt?: string;
  notes?: string;
  toolResults: ToolExecutionTelemetry[];
};

export type GoalAutomationState = {
  goalId: string;
  history: AgentAutomationRun[];
  lastUpdated: string;
  retryAvailable: boolean;
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
  automation?: GoalAutomationState | null;
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
