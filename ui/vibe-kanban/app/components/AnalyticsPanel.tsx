"use client";

import type { WorkspaceBoard } from "./board-types";

type AnalyticsPanelProps = {
  board: WorkspaceBoard | null;
};

function computeFlow(board: WorkspaceBoard | null) {
  if (!board) {
    return {
      throughput: 0,
      workInProgress: 0,
      vibeMomentum: 0,
    };
  }
  const doneColumn = board.columns.find((column) => /done|complete/i.test(column.title));
  const throughput = doneColumn?.cards.length ?? 0;
  const workInProgress = board.columns.reduce((count, column) => count + column.cards.length, 0) - throughput;
  const vibeMomentum = board.metrics?.vibeMomentum ?? Math.min(100, 40 + workInProgress * 5 - throughput * 3);
  return { throughput, workInProgress, vibeMomentum };
}

export function AnalyticsPanel({ board }: AnalyticsPanelProps) {
  const analytics = computeFlow(board);

  return (
    <div className="rounded-3xl border border-white/10 bg-surface/70 p-5 text-white/70">
      <h3 className="text-sm font-semibold uppercase tracking-[0.3em] text-white/50">Flow analytics</h3>
      <div className="mt-4 grid grid-cols-3 gap-4 text-center">
        <div className="rounded-2xl border border-white/10 bg-white/5 p-4">
          <div className="text-xs uppercase tracking-[0.3em] text-white/40">Throughput</div>
          <div className="mt-2 text-2xl font-semibold text-white">{analytics.throughput}</div>
        </div>
        <div className="rounded-2xl border border-white/10 bg-white/5 p-4">
          <div className="text-xs uppercase tracking-[0.3em] text-white/40">Work in play</div>
          <div className="mt-2 text-2xl font-semibold text-white">{analytics.workInProgress}</div>
        </div>
        <div className="rounded-2xl border border-white/10 bg-white/5 p-4">
          <div className="text-xs uppercase tracking-[0.3em] text-white/40">Momentum</div>
          <div className="mt-2 text-2xl font-semibold text-white">{analytics.vibeMomentum}%</div>
        </div>
      </div>
      <p className="mt-3 text-xs text-white/40">
        Unlock deeper analytics and billing dashboards by upgrading to the growth plan inside workspace settings.
      </p>
    </div>
  );
}
