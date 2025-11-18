"use client";

import type { WorkspaceIntegrationStatus } from "./board-types";

type IntegrationStatusProps = {
  integrations: WorkspaceIntegrationStatus[];
};

const statusStyles: Record<string, string> = {
  healthy: "border-emerald-400/40 bg-emerald-500/10 text-emerald-200",
  running: "border-sky-400/40 bg-sky-500/10 text-sky-200",
  degraded: "border-amber-400/40 bg-amber-500/10 text-amber-200",
  error: "border-rose-400/40 bg-rose-500/10 text-rose-200",
};

export function IntegrationStatus({ integrations }: IntegrationStatusProps) {
  return (
    <div className="rounded-3xl border border-white/10 bg-surface/70 p-5 text-white/70">
      <h3 className="text-sm font-semibold uppercase tracking-[0.3em] text-white/50">
        Platform signals
      </h3>
      <div className="mt-4 space-y-3">
        {integrations.map((integration) => (
          <div key={integration.id} className="rounded-2xl border border-white/10 bg-white/5 p-4">
            <div className="flex items-center justify-between">
              <div>
                <div className="text-sm font-semibold text-white">{integration.name}</div>
                <p className="text-xs text-white/40">{integration.lastEvent}</p>
              </div>
              <span
                className={`rounded-full border px-3 py-1 text-[11px] uppercase tracking-[0.2em] ${statusStyles[integration.status] ?? statusStyles.healthy}`}
              >
                {integration.status}
              </span>
            </div>
          </div>
        ))}
        {integrations.length === 0 && (
          <p className="text-sm text-white/40">
            No integrations yet — connect CRC, CI/CD, or runtime modules to light this up.
          </p>
        )}
      </div>
    </div>
  );
}
