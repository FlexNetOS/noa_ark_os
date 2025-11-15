"use client";

import type { ActivityEvent } from "./board-types";

function formatTimeAgo(iso: string) {
  const date = new Date(iso);
  const seconds = Math.floor((Date.now() - date.getTime()) / 1000);
  if (Number.isNaN(seconds)) return "just now";
  if (seconds < 60) return `${seconds}s ago`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

type ActivityTimelineProps = {
  activity: ActivityEvent[];
};

export function ActivityTimeline({ activity }: ActivityTimelineProps) {
  return (
    <div className="rounded-3xl border border-white/10 bg-surface/70 p-5 text-white/70">
      <h3 className="text-sm font-semibold uppercase tracking-[0.3em] text-white/50">Activity stream</h3>
      <div className="mt-4 space-y-4">
        {activity.slice(0, 8).map((event) => (
          <div key={event.id} className="flex items-start gap-3">
            <div className="mt-1 h-2.5 w-2.5 rounded-full bg-gradient-to-r from-purple-400 to-indigo-500" />
            <div>
              <p className="text-sm text-white/80">{event.description}</p>
              <p className="text-[11px] uppercase tracking-[0.2em] text-white/40">{formatTimeAgo(event.createdAt)}</p>
            </div>
          </div>
        ))}
        {activity.length === 0 && <p className="text-sm text-white/40">No moves yet — your actions will light up here.</p>}
      </div>
    </div>
  );
}
