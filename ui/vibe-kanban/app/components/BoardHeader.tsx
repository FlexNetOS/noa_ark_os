"use client";

import { useEffect, useMemo, useState } from "react";
import { AddColumnButton } from "./AddColumnButton";
import type { BoardMetrics } from "./board-types";

function formatDateLabel(iso: string) {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return "Just now";
  return date.toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
}

type BoardHeaderProps = {
  projectName: string;
  lastUpdated: string;
  onRename: (name: string) => void;
  onAddColumn: () => void;
  columnCount: number;
  totalCardCount: number;
  completedCount: number;
  showMetrics?: boolean;
  metrics?: BoardMetrics;
};

export function BoardHeader({
  projectName,
  lastUpdated,
  onRename,
  onAddColumn,
  columnCount,
  totalCardCount,
  completedCount,
  showMetrics = true,
  metrics: advancedMetrics,
}: BoardHeaderProps) {
  const [value, setValue] = useState(projectName);

  useEffect(() => {
    setValue(projectName);
  }, [projectName]);

  const metrics = useMemo(() => {
    const base: { label: string; value: string }[] = [
      { label: "Columns", value: columnCount.toString() },
      { label: "Active vibes", value: Math.max(totalCardCount - completedCount, 0).toString() },
      { label: "Completed", value: completedCount.toString() },
    ];
    if (advancedMetrics) {
      base.push({ label: "Vibe momentum", value: `${advancedMetrics.vibeMomentum}%` });
      if (advancedMetrics.cycleTimeDays) {
        base.push({ label: "Cycle time", value: `${advancedMetrics.cycleTimeDays}d` });
      }
      if (advancedMetrics.flowEfficiency) {
        base.push({ label: "Flow efficiency", value: `${advancedMetrics.flowEfficiency}%` });
      }
    }
    return base;
  }, [advancedMetrics, columnCount, completedCount, totalCardCount]);

  return (
    <div className="flex flex-col gap-6 rounded-[2.5rem] border border-white/10 bg-surface/70 p-8 backdrop-blur-xl">
      <div className="flex flex-col gap-6 lg:flex-row lg:items-center lg:justify-between">
        <div className="space-y-2">
          <span className="text-xs uppercase tracking-[0.4em] text-white/40">Workspace</span>
          <input
            value={value}
            onChange={(event) => setValue(event.target.value)}
            onBlur={() => onRename(value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.currentTarget.blur();
              }
            }}
            className="w-full max-w-xl bg-transparent text-3xl font-semibold tracking-tight text-white placeholder:text-white/40 focus:outline-none"
          />
          <p className="text-sm text-white/50">
            Last synced <time dateTime={lastUpdated}>{formatDateLabel(lastUpdated)}</time>
          </p>
        </div>
        <AddColumnButton onClick={onAddColumn} />
      </div>

      {showMetrics && (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
          {metrics.map((metric) => (
            <div
              key={metric.label}
              className="rounded-2xl border border-white/5 bg-white/5 p-4 text-white/80 transition hover:border-white/15 hover:bg-white/10"
            >
              <div className="text-xs uppercase tracking-[0.3em] text-white/40">{metric.label}</div>
              <div className="mt-2 text-2xl font-semibold text-white">{metric.value}</div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
