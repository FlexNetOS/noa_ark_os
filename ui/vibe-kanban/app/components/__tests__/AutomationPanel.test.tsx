import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { AutomationPanel } from "../AutomationPanel";
import type { VibeCard } from "../board-types";

const baseCard: VibeCard = {
  id: "card-1",
  title: "Prototype",
  notes: "",
  createdAt: new Date().toISOString(),
  mood: "focus",
};

describe("AutomationPanel", () => {
  it("renders placeholder when no automation data is present", () => {
    render(<AutomationPanel cards={[{ ...baseCard, automation: null }]} onRetry={vi.fn()} />);
    expect(screen.getByText(/No automation telemetry/i)).toBeInTheDocument();
  });

  it("displays latest run and tool results with retry button", async () => {
    const retry = vi.fn().mockResolvedValue(undefined);
    const cards: VibeCard[] = [
      {
        ...baseCard,
        automation: {
          goalId: "card-1",
          history: [
            {
              agentId: "agent-1",
              agentName: "Registry Scout",
              status: "failed",
              attempt: 1,
              startedAt: new Date().toISOString(),
              finishedAt: new Date().toISOString(),
              toolResults: [
                {
                  name: "Capability Scan",
                  capability: "workflow.toolReceipts",
                  status: "failed",
                  error: "missing capability",
                },
              ],
            },
          ],
          lastUpdated: new Date().toISOString(),
          retryAvailable: true,
        },
      },
    ];

    render(<AutomationPanel cards={cards} onRetry={retry} />);

    expect(screen.getByText("Registry Scout")).toBeInTheDocument();
    expect(screen.getByText(/Capability Scan/)).toBeInTheDocument();
    const retryButton = screen.getByRole("button", { name: /retry automation/i });
    fireEvent.click(retryButton);
    await waitFor(() => expect(retry).toHaveBeenCalledWith("card-1"));
  });
});
