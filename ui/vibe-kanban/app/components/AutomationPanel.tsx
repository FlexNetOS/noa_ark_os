"use client";

import { useMemo, useState } from "react";
import type { VibeCard, AgentAutomationRun } from "./board-types";
import { tokens } from "@noa-ark/shared-ui/tokens";

const STATUS_COLORS: Record<AgentAutomationRun["status"], string> = {
  queued: tokens.colors["status/info"],
  running: tokens.colors["status/warning"],
  completed: tokens.colors["status/success"],
  failed: tokens.colors["status/danger"],
};

export interface AutomationPanelProps {
  cards: VibeCard[];
  onRetry: (cardId: string) => Promise<void> | void;
}

export function AutomationPanel({ cards, onRetry }: AutomationPanelProps) {
  const [pending, setPending] = useState<Set<string>>(new Set());

  const entries = useMemo(() => {
    return cards
      .map((card) => ({
        card,
        latestRun: card.automation?.history?.[0] ?? null,
      }))
      .filter((entry) => entry.card.automation)
      .sort((a, b) => {
        const aTime = a.card.automation?.lastUpdated ?? "";
        const bTime = b.card.automation?.lastUpdated ?? "";
        return bTime.localeCompare(aTime);
      });
  }, [cards]);

  if (!entries.length) {
    return (
      <div style={{ padding: tokens.spacing.lg }}>
        <p style={{ color: tokens.colors["text/subtle"], fontSize: "0.9rem" }}>
          No automation telemetry captured yet. Trigger a workflow to begin tracking agent progress.
        </p>
      </div>
    );
  }

  async function handleRetry(cardId: string) {
    setPending((current) => new Set(current).add(cardId));
    try {
      await onRetry(cardId);
    } finally {
      setPending((current) => {
        const next = new Set(current);
        next.delete(cardId);
        return next;
      });
    }
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: tokens.spacing.md, padding: tokens.spacing.lg }}>
      {entries.map(({ card, latestRun }) => {
        const automation = card.automation!;
        const hasFailure = latestRun?.status === "failed";
        return (
          <div
            key={card.id}
            style={{
              borderRadius: tokens.radii.md,
              border: `1px solid ${tokens.colors["border/subtle"]}`,
              padding: tokens.spacing.md,
              background:
                latestRun?.status === "running"
                  ? tokens.colors["surface/glow"]
                  : tokens.colors["surface/primary"],
              display: "flex",
              flexDirection: "column",
              gap: tokens.spacing.sm,
            }}
          >
            <header style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <div>
                <p style={{ fontSize: "0.75rem", color: tokens.colors["text/subtle"], marginBottom: 4 }}>Goal</p>
                <strong style={{ fontSize: "1rem" }}>{card.title}</strong>
              </div>
              <div style={{ textAlign: "right" }}>
                <span
                  style={{
                    padding: `${tokens.spacing.xs} ${tokens.spacing.sm}`,
                    borderRadius: tokens.radii.full,
                    fontSize: "0.75rem",
                    fontWeight: 600,
                    color: tokens.colors["text/inverse"],
                    background: STATUS_COLORS[latestRun?.status ?? "queued"],
                  }}
                >
                  {latestRun?.status ?? "queued"}
                </span>
              </div>
            </header>

            {latestRun && (
              <section style={{ display: "flex", flexDirection: "column", gap: tokens.spacing.xs }}>
                <p style={{ fontSize: "0.8rem", color: tokens.colors["text/subtle"] }}>
                  Agent {latestRun.agentName} • Attempt {latestRun.attempt}
                </p>
                <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                  {latestRun.toolResults.map((tool) => (
                    <div
                      key={`${card.id}-${tool.capability}`}
                      style={{
                        display: "flex",
                        justifyContent: "space-between",
                        alignItems: "center",
                        borderRadius: tokens.radii.sm,
                        background: tokens.colors["surface/secondary"],
                        padding: `${tokens.spacing.xs} ${tokens.spacing.sm}`,
                      }}
                    >
                      <div>
                        <p style={{ fontSize: "0.8rem", fontWeight: 600 }}>{tool.name}</p>
                        <p style={{ fontSize: "0.75rem", color: tokens.colors["text/subtle"] }}>{tool.capability}</p>
                        {tool.error && (
                          <p style={{ fontSize: "0.75rem", color: tokens.colors["status/danger"] }}>{tool.error}</p>
                        )}
                        {tool.output && !tool.error && (
                          <p style={{ fontSize: "0.75rem", color: tokens.colors["text/muted"] }}>{tool.output}</p>
                        )}
                      </div>
                      <span style={{ fontSize: "0.75rem", textTransform: "capitalize" }}>{tool.status}</span>
                    </div>
                  ))}
                </div>
              </section>
            )}

            <footer style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <p style={{ fontSize: "0.75rem", color: tokens.colors["text/subtle"] }}>
                Updated {new Date(automation.lastUpdated).toLocaleString()}
              </p>
              <button
                type="button"
                onClick={() => void handleRetry(card.id)}
                disabled={pending.has(card.id) || (!automation.retryAvailable && !hasFailure)}
                style={{
                  borderRadius: tokens.radii.full,
                  border: `1px solid ${tokens.colors["border/strong"]}`,
                  padding: `${tokens.spacing.xs} ${tokens.spacing.md}`,
                  fontSize: "0.8rem",
                  fontWeight: 600,
                  color: hasFailure ? tokens.colors["status/danger"] : tokens.colors["text/primary"],
                  background: "transparent",
                  opacity: pending.has(card.id) ? 0.6 : 1,
                  cursor: pending.has(card.id) ? "progress" : "pointer",
                }}
              >
                {pending.has(card.id) ? "Retrying…" : hasFailure ? "Retry automation" : "Re-run"}
              </button>
            </footer>
          </div>
        );
      })}
    </div>
  );
}
