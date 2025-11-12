"use client";

import { useEffect } from "react";

import type { NotificationEvent } from "./board-types";

type NotificationCenterProps = {
  notifications: NotificationEvent[];
  onDismiss: (id: string) => void;
};

const severityStyles: Record<NotificationEvent["severity"], string> = {
  info: "from-sky-500 via-indigo-500 to-purple-500",
  success: "from-emerald-400 via-teal-400 to-cyan-400",
  warning: "from-amber-400 via-orange-400 to-rose-400",
  error: "from-rose-500 via-red-500 to-purple-500",
};

export function NotificationCenter({ notifications, onDismiss }: NotificationCenterProps) {
  useEffect(() => {
    if (!notifications.length) return;
    const timeout = setTimeout(() => {
      const [latest] = notifications;
      if (latest) onDismiss(latest.id);
    }, 5000);
    return () => clearTimeout(timeout);
  }, [notifications, onDismiss]);

  if (notifications.length === 0) return null;

  const [latest, ...rest] = notifications;

  return (
    <div className="pointer-events-none fixed bottom-6 right-6 flex flex-col gap-3">
      <div className="pointer-events-auto w-80 rounded-3xl border border-white/10 bg-surface/80 p-4 shadow-lg backdrop-blur-xl">
        <div className={`h-1 w-full rounded-full bg-gradient-to-r ${severityStyles[latest.severity]}`} />
        <div className="mt-3 text-sm font-semibold text-white">{latest.message}</div>
        <p className="text-xs uppercase tracking-[0.2em] text-white/40">
          {new Date(latest.createdAt).toLocaleTimeString()}
        </p>
      </div>
      {rest.slice(0, 2).map((notification) => (
        <div key={notification.id} className="pointer-events-none w-72 rounded-3xl border border-white/5 bg-surface/60 p-3 text-xs text-white/50 backdrop-blur">
          {notification.message}
        </div>
      ))}
    </div>
  );
}
