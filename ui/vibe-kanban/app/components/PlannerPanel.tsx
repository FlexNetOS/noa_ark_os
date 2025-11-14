"use client";

import type { PlannerPlan, PlannerState } from "./board-types";
import type { ResumeToken } from "@noa-ark/shared-ui/schema";

type PlannerPanelProps = {
  planner: PlannerState;
  onResume: (token: ResumeToken) => void;
};

export function PlannerPanel({ planner, onResume }: PlannerPanelProps) {
  const activePlans = planner.plans;
  const statusLabel = plannerStatusLabel(planner);

  return (
    <div className="rounded-3xl border border-white/10 bg-surface/70 p-5 text-white/70">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <h3 className="text-sm font-semibold uppercase tracking-[0.3em] text-white/50">Goal planner</h3>
          <p className="mt-1 text-xs text-white/40">Translate workspace goals into orchestrated workflow stages.</p>
        </div>
        <StatusBadge status={statusLabel} detail={planner.lastError} />
      </div>

      {activePlans.length === 0 ? (
        <p className="mt-4 text-sm text-white/40">Trigger assist to spin up a workflow-backed plan for your next goal.</p>
      ) : (
        <div className="mt-4 space-y-4">
          {activePlans.map((plan) => (
            <PlanCard key={plan.workflowId} plan={plan} plannerStatus={planner.status} onResume={onResume} />
          ))}
        </div>
      )}
    </div>
  );
}

function PlanCard({
  plan,
  plannerStatus,
  onResume,
}: {
  plan: PlannerPlan;
  plannerStatus: PlannerState["status"];
  onResume: (token: ResumeToken) => void;
}) {
  const stageOrder = plan.stages;
  const resumeAvailable = !!plan.resumeToken;
  const disableResume = plannerStatus === "planning" || !resumeAvailable;

  return (
    <div className="rounded-2xl border border-white/10 bg-white/5 p-4">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <p className="text-xs uppercase tracking-[0.3em] text-white/40">Goal</p>
          <h4 className="text-sm font-semibold text-white">{plan.goalTitle}</h4>
          <p className="text-xs text-white/50">Workflow {plan.workflowId}</p>
        </div>
        <div className="flex flex-col items-start gap-2 sm:items-end">
          <span className={statusChipClass(plan.status)}>{plan.status}</span>
          {plan.resumeToken && (
            <button
              type="button"
              onClick={() => onResume(plan.resumeToken as ResumeToken)}
              disabled={disableResume}
              className={`rounded-full border px-3 py-1 text-xs font-semibold transition ${
                disableResume
                  ? "cursor-not-allowed border-white/10 bg-white/5 text-white/40"
                  : "border-emerald-400/40 bg-emerald-500/10 text-emerald-200 hover:border-emerald-300/60 hover:bg-emerald-500/20"
              }`}
            >
              Resume workflow
            </button>
          )}
        </div>
      </div>
      <ul className="mt-3 space-y-2">
        {stageOrder.map((stage) => (
          <li key={stage.id} className="flex items-center justify-between rounded-xl border border-white/5 bg-white/5 px-3 py-2 text-xs">
            <div>
              <p className="font-semibold text-white">{stage.name}</p>
              <p className="text-[11px] uppercase tracking-[0.2em] text-white/40">Stage</p>
            </div>
            <span className={statusChipClass(stage.state)}>{stage.state}</span>
          </li>
        ))}
      </ul>
      <p className="mt-2 text-[11px] uppercase tracking-[0.2em] text-white/30">
        Updated {new Date(plan.updatedAt).toLocaleTimeString()}
      </p>
    </div>
  );
}

function plannerStatusLabel(planner: PlannerState) {
  if (planner.status === "error" && planner.lastError) {
    return "error";
  }
  return planner.status;
}

function StatusBadge({ status, detail }: { status: string; detail?: string }) {
  const colorClass =
    status === "planning"
      ? "text-amber-200/80"
      : status === "error"
        ? "text-rose-200/80"
        : "text-emerald-200/80";
  const label = status.charAt(0).toUpperCase() + status.slice(1);
  return (
    <div className="text-right">
      <p className={`text-[11px] uppercase tracking-[0.2em] ${colorClass}`}>{label}</p>
      {status === "error" && detail && <p className="mt-1 text-[11px] text-rose-200/70">{detail}</p>}
    </div>
  );
}

function statusChipClass(status: string) {
  switch (status) {
    case "running":
      return "rounded-full border border-sky-400/40 bg-sky-500/10 px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.2em] text-sky-200";
    case "completed":
      return "rounded-full border border-emerald-400/40 bg-emerald-500/10 px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.2em] text-emerald-200";
    case "failed":
      return "rounded-full border border-rose-400/40 bg-rose-500/10 px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.2em] text-rose-200";
    case "paused":
      return "rounded-full border border-amber-400/40 bg-amber-500/10 px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.2em] text-amber-200";
    default:
      return "rounded-full border border-white/10 bg-white/10 px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.2em] text-white/60";
  }
}
