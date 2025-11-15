import type { ResumeToken } from "@noa-ark/shared-ui/schema";

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

export type Goal = VibeCard;

export type VibeColumn = {
  id: string;
  title: string;
  accent: string;
  goals: Goal[];
  cards?: Goal[];
};

export type BoardMoodSample = {
  recordedAt: string;
  focus: number;
  flow: number;
  chill: number;
  hype: number;
};

export type BoardMetrics = {
  completedGoals: number;
  activeGoals: number;
  goalMomentum: number;
  cycleTimeDays?: number;
  flowEfficiency?: number;
  goalLeadTimeHours?: number;
  goalSuccessRate?: number;
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
  goalId?: string;
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

export type PlannerStage = {
  id: string;
  name: string;
  state: "pending" | "running" | "completed" | "failed" | "paused" | "skipped";
};

export type PlannerPlan = {
  goalId: string;
  goalTitle: string;
  workflowId: string;
  status: "pending" | "running" | "completed" | "failed" | "paused";
  stages: PlannerStage[];
  startedAt: string;
  updatedAt: string;
  resumeToken?: ResumeToken;
};

export type PlannerState = {
  status: "idle" | "ready" | "planning" | "error";
  plans: PlannerPlan[];
  activePlanId: string | null;
  lastError: string | null;
};
