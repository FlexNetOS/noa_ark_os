"use client";

import type { WorkspaceBoard } from "./board-types";

type AnalyticsPanelProps = {
  board: WorkspaceBoard | null;
  enableGoalInsights?: boolean;
};

function computeFlow(board: WorkspaceBoard | null) {
  if (!board) {
    return {
      throughput: 0,
      workInProgress: 0,
      vibeMomentum: 0,
      goalLeadTime: null as number | null,
      goalSuccessRate: null as number | null,
    };
  }
  const doneColumn = board.columns.find((column) => /done|complete/i.test(column.title));
  const throughput = doneColumn?.cards.length ?? 0;
  const workInProgress = board.columns.reduce((count, column) => count + column.cards.length, 0) - throughput;
  const vibeMomentum = board.metrics?.vibeMomentum ?? Math.min(100, 40 + workInProgress * 5 - throughput * 3);
  const goalLeadTime = typeof board.metrics?.goalLeadTimeHours === "number" ? board.metrics.goalLeadTimeHours : null;
  const goalSuccessRate = typeof board.metrics?.goalSuccessRate === "number" ? board.metrics.goalSuccessRate : null;
  return { throughput, workInProgress, vibeMomentum, goalLeadTime, goalSuccessRate };
}

export function AnalyticsPanel({ board, enableGoalInsights = false }: AnalyticsPanelProps) {
  const analytics = computeFlow(board);
  const cards: { label: string; value: string }[] = [
    { label: "Throughput", value: analytics.throughput.toString() },
    { label: "Work in play", value: analytics.workInProgress.toString() },
    { label: "Momentum", value: `${analytics.vibeMomentum}%` },
  ];

  const insightsActive = enableGoalInsights && (analytics.goalLeadTime !== null || analytics.goalSuccessRate !== null);

  if (insightsActive) {
    if (analytics.goalLeadTime !== null) {
      cards.push({ label: "Lead time", value: `${analytics.goalLeadTime}h` });
    }
    if (analytics.goalSuccessRate !== null) {
      cards.push({ label: "Goal success", value: `${analytics.goalSuccessRate}%` });
    }
  }

  return (
    <div className="rounded-3xl border border-white/10 bg-surface/70 p-5 text-white/70">
      <h3 className="text-sm font-semibold uppercase tracking-[0.3em] text-white/50">Flow analytics</h3>
      <div className={`mt-4 grid gap-4 text-center ${cards.length > 3 ? "grid-cols-2 md:grid-cols-4" : "grid-cols-3"}`}>
        {cards.map((card) => (
          <div key={card.label} className="rounded-2xl border border-white/10 bg-white/5 p-4">
            <div className="text-xs uppercase tracking-[0.3em] text-white/40">{card.label}</div>
            <div className="mt-2 text-2xl font-semibold text-white">{card.value}</div>
          </div>
        ))}
      </div>
      <p className="mt-3 text-xs text-white/40">
        {insightsActive
          ? "Goal-level KPIs refresh automatically when workflows complete."
          : "Unlock deeper analytics and billing dashboards by upgrading to the growth plan inside workspace settings."}
      </p>
    </div>
  );
}
