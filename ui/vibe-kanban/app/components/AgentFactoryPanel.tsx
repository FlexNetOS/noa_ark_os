"use client";

import type {
  GoalMemoryInsights,
  PlannerState,
  UploadReceiptSummary,
  WorkspaceBoard,
  WorkspaceIntegrationStatus,
  Workspace,
} from "./board-types";
import type { BoardState } from "./useBoardState";

type AgentFactoryContext = {
  autonomy: BoardState["autonomy"];
  goalInsights: GoalMemoryInsights | null;
  planner: PlannerState;
  assist: BoardState["assist"];
  snapshot: WorkspaceBoard | null;
  workspace: Workspace | null;
  uploadReceipts: UploadReceiptSummary[];
  integrations: WorkspaceIntegrationStatus[];
};

type AgentFactoryLayerStatus = {
  id: string;
  label: string;
  persona: string;
  state: "ready" | "alert";
  summary: string;
  signalLabel: string;
  signalValue: string;
  metricLabel: string;
  metricValue: string;
  escalation: string;
};

export function deriveAgentFactoryLayers(context: AgentFactoryContext): AgentFactoryLayerStatus[] {
  const boardMetrics = context.snapshot?.metrics;
  const successRate = boardMetrics?.goalSuccessRate;
  const leadTime = boardMetrics?.goalLeadTimeHours;
  const owner =
    context.workspace?.members.find((member) => member.role === "owner")?.name ?? "Workspace owner";
  const totalGoals =
    context.snapshot?.columns.reduce((count, column) => count + column.goals.length, 0) ?? 0;
  const activeColumns = context.snapshot?.columns.length ?? 0;
  const plannerPlan = context.planner.plans[0];
  const orchestratorFocus =
    context.assist?.focusGoal?.title ?? context.assist?.suggestions?.[0]?.title;
  const orchestrationMemory =
    context.goalInsights?.insightSummary ?? context.goalInsights?.summary ?? "Memory synced.";
  const degradedIntegrations = context.integrations.filter(
    (integration) => integration.status === "degraded" || integration.status === "error",
  );
  const lastUploadAt = latestUpload(context.uploadReceipts);

  return [
    {
      id: "L1",
      label: "L1 Autonomy",
      persona: "CECCA / Executive",
      state: context.autonomy.escalationTriggered ? "alert" : "ready",
      summary:
        context.autonomy.summary ??
        "Goal KPIs remain within guardrails; no escalations requested this cycle.",
      signalLabel: "Latest trigger",
      signalValue: formatRelativeTime(context.autonomy.lastTriggeredAt),
      metricLabel: "Lead time",
      metricValue: typeof leadTime === "number" ? `${leadTime.toFixed(1)}h` : "n/a",
      escalation: "Escalates to L2 Board when governance policies change.",
    },
    {
      id: "L2",
      label: "L2 Reasoning",
      persona: "Board Planners",
      state: context.planner.status === "error" ? "alert" : "ready",
      summary: plannerPlan
        ? `Tracking ${plannerPlan.goalTitle} via ${plannerPlan.workflowId}.`
        : "Planner is idle and ready for the next workspace plan.",
      signalLabel: "Queued plans",
      signalValue: `${context.planner.plans.length}`,
      metricLabel: "Active status",
      metricValue: plannerPlan?.status ?? context.planner.status,
      escalation: "Escalates to L3 Stack-Chiefs for orchestration hand-offs.",
    },
    {
      id: "L3",
      label: "L3 Orchestration",
      persona: "Stack-Chief",
      state: context.assist ? "ready" : "ready",
      summary: orchestratorFocus
        ? `Assist is monitoring "${orchestratorFocus}".`
        : "Assist is on standby for CRC drops or planner nudges.",
      signalLabel: "Memory traces",
      signalValue: context.goalInsights
        ? `${context.goalInsights.traceCount} captured`
        : "Telemetry syncing",
      metricLabel: "Insights",
      metricValue: truncate(orchestrationMemory, 42),
      escalation: "Escalates to L4 specialists if assistance stalls.",
    },
    {
      id: "L4",
      label: "L4 Operations",
      persona: "Specialists",
      state: typeof successRate === "number" && successRate < 60 ? "alert" : "ready",
      summary: `${totalGoals} in-flight goals across ${activeColumns} columns.`,
      signalLabel: "Success rate",
      signalValue: typeof successRate === "number" ? `${successRate}%` : "Waiting on signal",
      metricLabel: "Owner",
      metricValue: owner,
      escalation: "Escalates to L5 disposable agents for automated retries.",
    },
    {
      id: "L5",
      label: "L5 Infrastructure",
      persona: "Disposable agents",
      state: degradedIntegrations.length > 0 ? "alert" : "ready",
      summary:
        degradedIntegrations.length > 0
          ? `${degradedIntegrations.length} integration(s) require attention.`
          : "CRC bridge, CI/CD console, and runtime adapters are healthy.",
      signalLabel: "CRC drops",
      signalValue: lastUploadAt ? `Last drop ${formatRelativeTime(lastUploadAt)}` : "No drops yet",
      metricLabel: "Integrations",
      metricValue: `${context.integrations.length - degradedIntegrations.length}/${
        context.integrations.length || 1
      } healthy`,
      escalation: "Escalates back to L4 if automation cannot self-heal.",
    },
  ];
}

export function AgentFactoryPanel({ state }: { state: BoardState }) {
  const layers = deriveAgentFactoryLayers({
    autonomy: state.autonomy,
    goalInsights: state.goalInsights,
    planner: state.planner,
    assist: state.assist,
    snapshot: state.snapshot,
    workspace: state.workspace,
    uploadReceipts: state.uploadReceipts,
    integrations: state.integrations,
  });

  return (
    <div className="space-y-5 rounded-[3rem] border border-white/10 bg-gradient-to-b from-slate-950/80 to-slate-900/40 p-6 text-white shadow-[0_40px_120px_-60px_rgba(56,189,248,0.45)]">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <p className="text-xs uppercase tracking-[0.5em] text-white/40">Agent Factory</p>
          <h2 className="mt-2 text-2xl font-semibold">Governance & Escalation</h2>
          <p className="mt-2 max-w-2xl text-sm text-white/60">
            Maps Kanban personas to the L1–L5 hierarchy from the Agent Factory (see{" "}
            <a
              className="underline decoration-dotted hover:text-sky-200"
              href="agents/README.md#5-layer-agent-hierarchy"
            >
              agents/README.md
            </a>
            ) so shell telemetry, escalations, and kill switches remain constitutional.
          </p>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        {layers.map((layer) => (
          <div
            key={layer.id}
            className="rounded-[2rem] border border-white/10 bg-white/5 p-5 backdrop-blur"
          >
            <div className="flex items-start justify-between">
              <div>
                <p className="text-[0.55rem] uppercase tracking-[0.5em] text-white/40">
                  {layer.id}
                </p>
                <h3 className="text-lg font-semibold text-white">{layer.label}</h3>
                <p className="text-xs uppercase tracking-[0.3em] text-white/45">{layer.persona}</p>
              </div>
              <span
                className={`rounded-full px-3 py-1 text-xs font-semibold ${
                  layer.state === "alert"
                    ? "bg-amber-500/20 text-amber-200"
                    : "bg-emerald-500/20 text-emerald-200"
                }`}
              >
                {layer.state === "alert" ? "Attention" : "Steady"}
              </span>
            </div>

            <p className="mt-4 text-sm text-white/70">{layer.summary}</p>

            <dl className="mt-5 grid grid-cols-2 gap-4 text-xs text-white/60">
              <div>
                <dt className="uppercase tracking-[0.3em] text-white/40">{layer.signalLabel}</dt>
                <dd className="mt-1 text-sm font-semibold text-white">{layer.signalValue}</dd>
              </div>
              <div>
                <dt className="uppercase tracking-[0.3em] text-white/40">{layer.metricLabel}</dt>
                <dd className="mt-1 text-sm font-semibold text-white">{layer.metricValue}</dd>
              </div>
            </dl>

            <p className="mt-4 text-xs text-white/50">{layer.escalation}</p>
          </div>
        ))}
      </div>
    </div>
  );
}

function formatRelativeTime(iso: string | null | undefined): string {
  if (!iso) return "n/a";
  const parsed = Date.parse(iso);
  if (Number.isNaN(parsed)) return "n/a";
  const diffMs = Date.now() - parsed;
  const absMinutes = Math.floor(Math.abs(diffMs) / 60000);
  if (absMinutes < 60) {
    return `${absMinutes}m ago`;
  }
  const absHours = Math.floor(absMinutes / 60);
  return `${absHours}h ago`;
}

function latestUpload(receipts: UploadReceiptSummary[]): string | null {
  if (receipts.length === 0) return null;
  return receipts.reduce((latest, receipt) => {
    if (!latest) return receipt.uploadedAt;
    return Date.parse(receipt.uploadedAt) > Date.parse(latest) ? receipt.uploadedAt : latest;
  }, receipts[0]?.uploadedAt ?? null);
}

function truncate(text: string, max: number) {
  if (text.length <= max) {
    return text;
  }
  return `${text.slice(0, max - 1)}…`;
}
