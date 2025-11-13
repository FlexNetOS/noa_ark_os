"use client";

import { useEffect, useMemo, useState } from "react";

import type { CapabilityFeatureGateStatus } from "@/shared/capabilities";

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
  canAddColumn?: boolean;
  addColumnDisabledReason?: string;
  columnCount: number;
  totalCardCount: number;
  completedCount: number;
  showMetrics?: boolean;
  metrics?: BoardMetrics;
  goalInsightsEnabled?: boolean;
  capabilitySummary?: CapabilityFeatureGateStatus[];
  capabilitiesLoading?: boolean;
};

export function BoardHeader({
  projectName,
  lastUpdated,
  onRename,
  onAddColumn,
  canAddColumn = true,
  addColumnDisabledReason,
  columnCount,
  totalCardCount,
  completedCount,
  showMetrics = true,
  metrics: advancedMetrics,
  goalInsightsEnabled = false,
  capabilitySummary = [],
  capabilitiesLoading = false,
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
      if (goalInsightsEnabled && typeof advancedMetrics.goalLeadTimeHours === "number") {
        base.push({ label: "Lead time", value: `${advancedMetrics.goalLeadTimeHours}h` });
      }
      if (goalInsightsEnabled && typeof advancedMetrics.goalSuccessRate === "number") {
        base.push({ label: "Goal success", value: `${advancedMetrics.goalSuccessRate}%` });
      }
    }
    return base;
  }, [advancedMetrics, columnCount, completedCount, goalInsightsEnabled, totalCardCount]);

  const hasCapabilitySummary = capabilitySummary.length > 0;

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
        <AddColumnButton
          onClick={onAddColumn}
          disabled={!canAddColumn}
          disabledReason={addColumnDisabledReason}
        />
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

      {hasCapabilitySummary && (
        <CapabilitySummaryPanel
          items={capabilitySummary}
          loading={capabilitiesLoading}
        />
      )}
    </div>
  );
}

type CapabilitySummaryPanelProps = {
  items: CapabilityFeatureGateStatus[];
  loading: boolean;
};

function CapabilitySummaryPanel({ items, loading }: CapabilitySummaryPanelProps) {
  return (
    <div className="rounded-2xl border border-white/5 bg-white/5 p-5 text-white/80">
      <div className="flex items-center justify-between gap-3">
        <div>
          <div className="text-xs uppercase tracking-[0.3em] text-white/40">Capability summary</div>
          <p className="mt-1 text-xs text-white/50">
            Features adjust automatically based on the active registry.
          </p>
        </div>
        <span className="rounded-full border border-white/10 bg-white/10 px-3 py-1 text-[11px] uppercase tracking-[0.2em] text-white/60">
          {loading ? "Syncing" : "Updated"}
        </span>
      </div>
      <ul className="mt-4 space-y-3">
        {items.map((item) => {
          const statusLabel = loading
            ? "Pending"
            : item.available
              ? "Available"
              : "Unavailable";
          const badgeClasses = loading
            ? "border-white/20 bg-white/5 text-white/50"
            : item.available
              ? "border-emerald-400/60 bg-emerald-500/20 text-emerald-100"
              : "border-amber-400/40 bg-amber-500/10 text-amber-100";
          const icon = loading ? "…" : item.available ? "✓" : "!";
          return (
            <li
              key={item.id}
              data-testid={`capability-${item.id}`}
              className="flex items-start gap-3 rounded-2xl border border-white/10 bg-surface/70 p-4"
            >
          <span
            aria-hidden
            className={`mt-0.5 flex h-6 w-6 items-center justify-center rounded-full border text-sm font-semibold ${badgeClasses}`}
          >
            {icon}
          </span>
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <span className="text-sm font-semibold text-white">{item.label}</span>
              <span className={`text-xs font-semibold uppercase tracking-[0.2em] ${
                loading
                  ? "text-white/60"
                  : item.available
                    ? "text-emerald-200"
                    : "text-amber-200"
              }`}
              >
                {statusLabel}
              </span>
            </div>
            <p className="text-xs text-white/60">{item.description}</p>
            {!loading && !item.available && (
              <p className="text-[11px] uppercase tracking-[0.2em] text-amber-200/80">
                Requires capability token: {item.capability}
              </p>
            )}
            </div>
          </li>
          );
        })}
      </ul>
    </div>
  );
}
