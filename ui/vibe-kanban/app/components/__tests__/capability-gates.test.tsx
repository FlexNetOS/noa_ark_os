import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { AssistPanel } from "../AssistPanel";
import { BoardHeader } from "../BoardHeader";

const baseCapability = {
  label: "Column management",
  description: "Create, rename, and remove workflow columns.",
};

describe("capability-gated UI", () => {
  it("disables add column and surfaces capability summary when the feature is unavailable", () => {
    render(
      <BoardHeader
        projectName="Studio Board"
        lastUpdated="2024-05-01T00:00:00.000Z"
        onRename={vi.fn()}
        onAddColumn={vi.fn()}
        canAddColumn={false}
        addColumnDisabledReason="Enable capability"
        columnCount={3}
        totalGoalCount={9}
        completedGoalCount={2}
        showMetrics={false}
        capabilitySummary={[
          {
            id: "kanban.manageColumns",
            capability: "kanban.manageColumns",
            available: false,
            ...baseCapability,
          },
        ]}
        capabilitiesLoading={false}
      />
    );

    const button = screen.getByRole("button", { name: /add column/i });
    expect(button.getAttribute("disabled")).not.toBeNull();
    expect(button.getAttribute("title")).toMatch(/Enable capability/i);
    expect(screen.getByTestId("capability-kanban.manageColumns").textContent).toMatch(/unavailable/i);
  });

  it("gates the assist panel when the capability is missing and re-enables once restored", () => {
    const onRequest = vi.fn();
    const { rerender } = render(
      <AssistPanel
        assist={null}
        onRequest={onRequest}
        capability={{
          id: "kanban.assist",
          capability: "kanban.assist",
          label: "Spark assist agent",
          description: "Request AI suggestions tailored to the active board.",
          available: false,
        }}
        loading={false}
      />
    );

    const disabledButton = screen.getByRole("button", { name: /spark assist/i });
    expect(disabledButton.getAttribute("disabled")).not.toBeNull();
    expect(screen.getByTestId("assist-empty-message").textContent).toMatch(/enable the kanban\.assist capability/i);

    rerender(
      <AssistPanel
        assist={null}
        onRequest={onRequest}
        capability={{
          id: "kanban.assist",
          capability: "kanban.assist",
          label: "Spark assist agent",
          description: "Request AI suggestions tailored to the active board.",
          available: true,
        }}
        loading={false}
      />
    );

    const enabledButton = screen.getByRole("button", { name: /spark assist/i });
    expect(enabledButton.disabled).toBe(false);
    expect(screen.getByTestId("assist-capability-status").textContent).toMatch(/ready/i);
  });
});
