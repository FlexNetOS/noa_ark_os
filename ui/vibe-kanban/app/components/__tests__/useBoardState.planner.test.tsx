import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { ResumeToken } from "@noa-ark/shared-ui/schema";
import { useBoardState } from "../useBoardState";
import type { WorkspaceBoard } from "../board-types";
import * as sessionModule from "@noa-ark/shared-ui/session";

type Listener = (value: unknown) => void;

function createSessionContinuityMock() {
  class MockSessionContinuityClient {
    static last: MockSessionContinuityClient | null = null;
    private listeners = new Map<string, Set<Listener>>();
    public requested: ResumeToken[] = [];
    private connected = false;

    constructor() {
      MockSessionContinuityClient.last = this;
    }

    on(event: string, handler: Listener) {
      const set = this.listeners.get(event) ?? new Set<Listener>();
      set.add(handler);
      this.listeners.set(event, set);
    }

    off(event: string, handler: Listener) {
      const set = this.listeners.get(event);
      set?.delete(handler);
      if (set && set.size === 0) {
        this.listeners.delete(event);
      }
    }

    connectWebSocket() {
      this.connected = true;
    }

    disconnect() {
      this.connected = false;
      this.listeners.clear();
      if (MockSessionContinuityClient.last === this) {
        MockSessionContinuityClient.last = null;
      }
    }

    requestResume(token: ResumeToken) {
      this.requested.push(token);
    }

    emit(event: string, payload: unknown) {
      if (!this.connected) {
        this.connectWebSocket();
      }
      const set = this.listeners.get(event);
      set?.forEach((handler) => handler(payload));
    }
  }

  return {
    SessionContinuityClient: MockSessionContinuityClient,
    getLastClient: () => MockSessionContinuityClient.last,
  };
}

vi.mock("@noa-ark/shared-ui/session", async () => {
  const actual = await vi.importActual<typeof import("@noa-ark/shared-ui/session")>(
    "@noa-ark/shared-ui/session",
  );
  const { SessionContinuityClient, getLastClient } = createSessionContinuityMock();

  return {
    ...actual,
    SessionContinuityClient,
    __getLastClient: getLastClient,
  };
});

const fetchMock = vi.fn();

global.fetch = fetchMock as unknown as typeof fetch;
type MockMessageEvent = MessageEvent & { data: string };
type MockEventListener = (event: MockMessageEvent) => void;

class MockEventSource {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSED = 2;

  url: string;
  readonly CONNECTING = MockEventSource.CONNECTING;
  readonly OPEN = MockEventSource.OPEN;
  readonly CLOSED = MockEventSource.CLOSED;
  readyState = MockEventSource.CONNECTING;
  onopen: ((this: EventSource, ev: Event) => unknown) | null = null;
  onmessage: ((this: EventSource, ev: MessageEvent) => unknown) | null = null;
  onerror: ((this: EventSource, ev: Event) => unknown) | null = null;
  withCredentials = false;
  private listeners = new Map<string, Set<MockEventListener>>();

  constructor(url: string) {
    this.url = url;
    this.readyState = MockEventSource.OPEN;
  }

  addEventListener(event: string, listener: MockEventListener) {
    const handlers = this.listeners.get(event) ?? new Set<MockEventListener>();
    handlers.add(listener);
    this.listeners.set(event, handlers);
  }

  removeEventListener(event: string, listener: MockEventListener) {
    const handlers = this.listeners.get(event);
    handlers?.delete(listener);
    if (handlers && handlers.size === 0) {
      this.listeners.delete(event);
    }
  }

  dispatchEvent(_: Event): boolean {
    return true;
  }

  emit(event: string, data: string) {
    const payload = { data, type: event } as MockMessageEvent;
    this.listeners.get(event)?.forEach((listener) => listener(payload));
    if (event === "message") {
      this.onmessage?.call(this as unknown as EventSource, payload);
    }
  }

  close() {
    this.readyState = MockEventSource.CLOSED;
    this.listeners.clear();
    this.onopen = null;
    this.onmessage = null;
    this.onerror = null;
  }
}

(globalThis as { EventSource?: typeof EventSource }).EventSource =
  MockEventSource as unknown as typeof EventSource;

type MockClientShape = {
  requested: ResumeToken[];
  emit: (event: string, payload: unknown) => void;
  disconnect: () => void;
};

type SessionModuleWithMock = typeof sessionModule & {
  __getLastClient?: () => MockClientShape | null;
};

const getLastClient = () => (sessionModule as SessionModuleWithMock).__getLastClient?.() ?? null;

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
      },
      {
        id: "in-progress",
        title: "In Progress",
        accent: "from-sky-500 via-cyan-400 to-emerald-400",
        goals: [],
      },
      {
        id: "done",
        title: "Done",
        accent: "from-violet-500 via-indigo-400 to-fuchsia-500",
        goals: [],
      },
    ],
    lastUpdated: new Date().toISOString(),
  };

  beforeEach(() => {
    fetchMock.mockImplementation(async (input: RequestInfo) => {
      const url = typeof input === "string" ? input : input.url;
      if (url.endsWith("/api/workspaces")) {
        return {
          ok: true,
          json: async () => ({
            workspaces: [
              {
                id: "studio",
                name: "Studio",
                accent: "from-indigo-500 via-purple-500 to-blue-500",
                createdAt: new Date().toISOString(),
                billingPlan: "starter",
                members: [],
                boards: [board],
                activity: [],
                notifications: [],
                uploadReceipts: [],
              },
            ],
          }),
        } as Response;
      }
      if (url.endsWith("/api/workspaces/studio")) {
        return {
          ok: true,
          json: async () => ({
            workspace: {
              id: "studio",
              members: [],
              boards: [board],
              activity: [],
              notifications: [],
              uploadReceipts: [],
            },
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
            focusCard: board.columns[0].goals[0],
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
              stages: [{ id: "goal-intake", name: "Goal Intake", state: "pending" }],
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
      client?.emit("workflow:update", {
        eventType: "workflow/stage",
        workflowId: plan.workflowId,
        payload: { stage_id: "Goal Intake", state: "Running" },
        timestamp: new Date().toISOString(),
      });
    });

    await waitFor(() => expect(result.current.planner.plans[0].stages[0].state).toBe("running"));

    const beforeRefresh = fetchMock.mock.calls.filter(([input]) =>
      String(input).endsWith("/api/workspaces/studio/boards/launchpad"),
    ).length;

    await act(async () => {
      client?.emit("workflow:update", {
        eventType: "workflow/state",
        workflowId: plan.workflowId,
        payload: { state: "Completed" },
        timestamp: new Date().toISOString(),
      });
    });

    await waitFor(() => expect(result.current.planner.plans[0].status).toBe("completed"));

    const afterRefresh = fetchMock.mock.calls.filter(([input]) =>
      String(input).endsWith("/api/workspaces/studio/boards/launchpad"),
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
