import React from "react";
import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";

import type { Goal } from "../../../components/board-types";
import { AIPromptButton } from "../AIPromptButton";

const exampleGoal: Goal = {
  id: "goal-42",
  title: "Wire gateway observability",
  notes: "Add dashboards for latency and error rate.",
  createdAt: new Date("2024-05-01T00:00:00Z").toISOString(),
  mood: "focus",
  integrations: [{ kind: "runtime", label: "Gateway", status: "success" }],
};

describe("AIPromptButton", () => {
  const originalFetch = global.fetch;
  const originalClipboard = navigator.clipboard;

  beforeEach(() => {
    global.fetch = vi.fn(
      async () =>
        new Response(
          JSON.stringify({ prompt: "Feature: Wire gateway observability", completion: null }),
          { status: 200, headers: { "Content-Type": "application/json" } },
        ),
    );
    Object.assign(navigator, {
      clipboard: {
        writeText: vi.fn().mockResolvedValue(undefined),
      },
    });
  });

  afterEach(() => {
    global.fetch = originalFetch;
    Object.assign(navigator, { clipboard: originalClipboard });
    vi.restoreAllMocks();
  });

  it("requests a prompt and enables copy", async () => {
    render(<AIPromptButton goal={exampleGoal} />);
    const button = screen.getByRole("button", { name: /generate ai implementation prompt/i });
    await act(async () => {
      await userEvent.click(button);
    });

    await screen.findByText(/Feature: Wire gateway observability/);

    expect(global.fetch).toHaveBeenCalledWith(
      "/api/ai/prompt",
      expect.objectContaining({
        method: "POST",
        body: expect.stringContaining('"goal-42"'),
      }),
    );

    const copyButton = screen.getByRole("button", { name: /copy/i });
    await act(async () => {
      await userEvent.click(copyButton);
    });

    await waitFor(() => {
      expect(navigator.clipboard.writeText).toHaveBeenCalledWith(
        expect.stringContaining("Feature: Wire gateway observability"),
      );
    });

    expect(screen.getByText(/Prompt copied to clipboard/)).toBeTruthy();
  });
});
