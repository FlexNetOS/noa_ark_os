"use client";

import type { GoalMemoryInsights } from "./board-types";

type PlannerPanelProps = {
  insights: GoalMemoryInsights | null;
  onRefresh: () => Promise<void>;
  loading?: boolean;
};

export function PlannerPanel({ insights, onRefresh, loading = false }: PlannerPanelProps) {
  const summary = insights?.summary ?? "Memory is warming up. Trigger assist to populate long-term context.";
  return (
    <div className="rounded-3xl border border-white/10 bg-surface/70 p-5 text-white/70">
      <div className="flex items-center justify-between gap-4">
        <div>
          <h3 className="text-sm font-semibold uppercase tracking-[0.3em] text-white/50">Planner memory</h3>
          <p className="mt-1 text-xs text-white/40">Surface historical traces and similar missions to inform the next plan.</p>
        </div>
        <button
          onClick={() => onRefresh()}
          disabled={loading}
          className={`rounded-full border px-3 py-1 text-[11px] font-semibold transition ${
            loading
              ? "cursor-not-allowed border-white/10 bg-white/5 text-white/40"
              : "border-sky-400/40 bg-sky-500/10 text-sky-100 hover:border-sky-300/60 hover:bg-sky-500/20"
          }`}
        >
          {loading ? "Syncing…" : "Refresh"}
        </button>
      </div>
      <p className="mt-3 text-xs text-white/60">{summary}</p>
      {insights ? (
        <div className="mt-4 space-y-3 text-xs text-white/60">
          <div className="rounded-2xl border border-white/10 bg-white/5 p-4">
            <div className="text-xs uppercase tracking-[0.3em] text-white/40">Signals</div>
            <p className="mt-2 text-sm text-white/70">
              {insights.insightSummary ?? `${insights.traceCount} traces with ${insights.similarGoals.length} similar goals indexed.`}
            </p>
          </div>
          {insights.similarGoals.length > 0 && (
            <div className="rounded-2xl border border-white/10 bg-white/5 p-4">
              <div className="text-xs uppercase tracking-[0.3em] text-white/40">Nearest goals</div>
              <ul className="mt-2 space-y-2">
                {insights.similarGoals.slice(0, 3).map((goal) => (
                  <li key={goal.goalId} className="flex items-center justify-between text-[11px]">
                    <span className="font-semibold text-white/80">{goal.goalId}</span>
                    <span className="text-white/50">{goal.score.toFixed(2)}</span>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      ) : null}
    </div>
  );
}
