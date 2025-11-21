import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { ResumeToken } from "@noa-ark/shared-ui/schema";
import { useBoardState } from "../useBoardState";
import type { WorkspaceBoard } from "../board-types";
import * as sessionModule from "@noa-ark/shared-ui/session";

vi.mock("@noa-ark/shared-ui/session", async () => {
  const actual = await vi.importActual<typeof import("@noa-ark/shared-ui/session")>("@noa-ark/shared-ui/session");

  class MockSessionContinuityClient {
    static last: MockSessionContinuityClient | null = null;
    private listeners = new Map<string, Set<(value: unknown) => void>>();
    public requested: ResumeToken[] = [];
    private connected = false;

    constructor() {
      MockSessionContinuityClient.last = this;
    }

    on(event: string, handler: (value: unknown) => void) {
      const set = this.listeners.get(event) ?? new Set();
      set.add(handler);
      this.listeners.set(event, set);
    }

    off(event: string, handler: (value: unknown) => void) {
      const set = this.listeners.get(event);
      set?.delete(handler);
    }

    connectWebSocket() {
      this.connected = true;
      this.emit("connection:open", undefined);
    }

    disconnect() {
      if (!this.connected) return;
      this.connected = false;
      this.emit("connection:closed", undefined);
    }

    requestResume(token: ResumeToken) {
      this.requested.push(token);
    }

    emit(event: string, payload?: unknown) {
      const set = this.listeners.get(event);
      set?.forEach((handler) => handler(payload));
    }
  }

  return {
    ...actual,
    SessionContinuityClient: MockSessionContinuityClient,
    __getLastClient: () => MockSessionContinuityClient.last,
  };
});

const fetchMock = vi.fn();

global.fetch = fetchMock as unknown as typeof fetch;

type LastClientAccessor = { __getLastClient?: () => unknown };
type LastClientLike = { disconnect?: () => void; emit?: (event: string, payload: unknown) => void; requested?: unknown[] };
const getLastClient = () =>
  (sessionModule as unknown as LastClientAccessor).__getLastClient?.() as LastClientLike | undefined;

const NOW = new Date().toISOString();

describe("useBoardState planner integration", () => {
  const board: WorkspaceBoard = {
    id: "launchpad",
    workspaceId: "studio",
    projectName: "Vibe Coding Launchpad",
    columns: [
      {
        id: "todo",
        title: "To Do",
        accent: "from-indigo-500 via-purple-500 to-blue-500",
        goals: [
          {
            id: "card-1",
            title: "Prototype kanban drag",
            notes: "Polish easing curve + inertia for drag transitions.",
            createdAt: new Date().toISOString(),
            mood: "focus",
          },
        ],
        cards: [
          {
            id: "card-1",
            title: "Prototype kanban drag",
            notes: "Polish easing curve + inertia for drag transitions.",
            createdAt: new Date().toISOString(),
            mood: "focus",
          },
        ],
      },
      {
        id: "in-progress",
        title: "In Progress",
        accent: "from-sky-500 via-cyan-400 to-emerald-400",
        goals: [],
        cards: [],
      },
      {
        id: "done",
        title: "Done",
        accent: "from-violet-500 via-indigo-400 to-fuchsia-500",
        goals: [],
        cards: [],
      },
    ],
    lastUpdated: new Date().toISOString(),
  };

  const workspace = {
    id: "studio",
    name: "Studio",
    accent: "from-indigo-500 via-purple-500 to-blue-500",
    createdAt: NOW,
    billingPlan: "starter",
    members: [],
    boards: [board],
    activity: [],
    notifications: [],
    uploadReceipts: [],
  };

  beforeEach(() => {
    fetchMock.mockImplementation(async (input: RequestInfo) => {
      const url = typeof input === "string" ? input : input.url;
      if (url.endsWith("/api/workspaces")) {
        return { ok: true, json: async () => ({ workspaces: [workspace] }) } as Response;
      }
      if (url.endsWith("/api/workspaces/studio")) {
        return {
          ok: true,
          json: async () => ({
            workspace,
          }),
        } as Response;
      }
      if (url.endsWith("/api/workspaces/studio/boards/launchpad")) {
        return { ok: true, json: async () => ({ board }) } as Response;
      }
      if (url.endsWith("/api/workspaces/studio/integrations/status")) {
        return { ok: true, json: async () => ({ integrations: [] }) } as Response;
      }
      if (url.endsWith("/api/capabilities")) {
        return { ok: true, json: async () => ({ version: "1", capabilities: [] }) } as Response;
      }
      if (url.endsWith("/assist")) {
        return {
          ok: true,
          json: async () => ({
            suggestions: [],
            focusCard: board.columns[0]!.goals[0]!,
            plan: {
              goalId: "goal-1",
              goalTitle: "Prototype kanban drag",
              workflowId: "goal-1",
              state: "pending",
              resumeToken: {
                workflowId: "goal-1",
                stageId: "goal-intake",
                checkpoint: "stage://goal-1/goal-intake",
                issuedAt: new Date().toISOString(),
                expiresAt: new Date(Date.now() + 1000 * 60 * 60).toISOString(),
              },
              startedAt: new Date().toISOString(),
              stages: [
                { id: "goal-intake", name: "Goal Intake", state: "pending" },
              ],
            },
          }),
        } as Response;
      }
      return { ok: true, json: async () => ({}) } as Response;
    });
  });

  afterEach(() => {
    fetchMock.mockReset();
    const client = getLastClient();
    client?.disconnect?.();
  });

  it("updates planner state from workflow events", async () => {
    const user = { id: "ava", name: "Ava" };
    const { result } = renderHook(() => useBoardState(user));

    await waitFor(() => expect(result.current.hydrated).toBe(true));

    await act(async () => {
      await result.current.requestAssist();
    });

    expect(result.current.planner.plans).toHaveLength(1);
    const plan = result.current.planner.plans[0];
    expect(plan.stages[0].state).toBe("pending");

    const client = getLastClient();
    await act(async () => {
      client?.emit?.("workflow:update", {
        eventType: "workflow/stage",
        workflowId: plan.workflowId,
        payload: { stage_id: "Goal Intake", state: "Running" },
        timestamp: new Date().toISOString(),
      });
    });

    await waitFor(() => expect(result.current.planner.plans[0].stages[0].state).toBe("running"));

    const beforeRefresh = fetchMock.mock.calls.filter(([input]) =>
      String(input).endsWith("/api/workspaces/studio/boards/launchpad")
    ).length;

    await act(async () => {
      client?.emit?.("workflow:update", {
        eventType: "workflow/state",
        workflowId: plan.workflowId,
        payload: { state: "Completed" },
        timestamp: new Date().toISOString(),
      });
    });

    await waitFor(() => expect(result.current.planner.plans[0].status).toBe("completed"));

    const afterRefresh = fetchMock.mock.calls.filter(([input]) =>
      String(input).endsWith("/api/workspaces/studio/boards/launchpad")
    ).length;
    expect(afterRefresh).toBeGreaterThan(beforeRefresh);

    const token = plan.resumeToken as ResumeToken;
    act(() => {
      result.current.resumePlan(token);
    });

    const recorded = getLastClient()?.requested ?? [];
    expect(recorded[recorded.length - 1]).toEqual(token);
  });
});
