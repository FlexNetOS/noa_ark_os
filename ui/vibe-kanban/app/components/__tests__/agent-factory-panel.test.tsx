import { describe, expect, it } from "vitest";

import { deriveAgentFactoryLayers } from "../AgentFactoryPanel";
import type { GoalMemoryInsights, PlannerState, WorkspaceBoard } from "../board-types";

type Context = Parameters<typeof deriveAgentFactoryLayers>[0];

const ISO = "2024-06-01T12:00:00.000Z";

const baseGoalInsights: GoalMemoryInsights = {
  summary: "Memory steady",
  traceCount: 2,
  lastSeen: ISO,
  traces: [],
  lifecycle: [],
  artifacts: [],
  similarGoals: [],
  insightSummary: "Memory steady",
  updatedAt: ISO,
};

const basePlanner: PlannerState = {
  status: "ready",
  plans: [],
  activePlanId: null,
  lastError: null,
};

const baseBoard: WorkspaceBoard = {
  id: "b",
  workspaceId: "ws",
  columns: [],
  lastUpdated: ISO,
  projectName: "Demo",
  metrics: {
    completedGoals: 0,
    activeGoals: 0,
    goalMomentum: 0,
  },
};

function createContext(partial: Partial<Context> = {}): Context {
  return {
    autonomy: {
      replanTriggered: false,
      escalationTriggered: false,
      lastTriggeredAt: ISO,
      summary: null,
      ...(partial.autonomy ?? {}),
    },
    goalInsights: partial.goalInsights ?? baseGoalInsights,
    planner: {
      ...basePlanner,
      ...(partial.planner ?? {}),
    },
    assist: partial.assist ?? null,
    snapshot: partial.snapshot ?? baseBoard,
    workspace:
      partial.workspace ??
      ({
        id: "ws",
        name: "Ops",
        accent: "#fff",
        createdAt: ISO,
        billingPlan: "growth",
        members: [
          {
            id: "owner",
            name: "Kai",
            role: "owner",
            avatarHue: 200,
          },
        ],
        boards: [],
        activity: [],
        notifications: [],
        uploadReceipts: [],
      } as Context["workspace"]),
    uploadReceipts: partial.uploadReceipts ?? [],
    integrations: partial.integrations ?? [],
  };
}

describe("deriveAgentFactoryLayers", () => {
  it("flags L1 when escalations fire", () => {
    const layers = deriveAgentFactoryLayers(
      createContext({
        autonomy: {
          replanTriggered: false,
          escalationTriggered: true,
          lastTriggeredAt: ISO,
          summary: "Escalation requested",
        },
      }),
    );
    const l1 = layers.find((layer) => layer.id === "L1");
    expect(l1?.state).toBe("alert");
    expect(l1?.summary).toContain("Escalation");
  });

  it("marks L4 operations as alert when success rate dips", () => {
    const board: WorkspaceBoard = {
      ...baseBoard,
      columns: [
        {
          id: "doing",
          title: "Doing",
          accent: "hue",
          goals: [{ id: "g", title: "Goal", notes: "", createdAt: ISO, mood: "flow" }],
        },
      ],
      metrics: {
        completedGoals: 1,
        activeGoals: 1,
        goalMomentum: 4,
        goalSuccessRate: 42,
      },
    };
    const layers = deriveAgentFactoryLayers(createContext({ snapshot: board }));
    const l4 = layers.find((layer) => layer.id === "L4");
    expect(l4?.state).toBe("alert");
    expect(l4?.signalValue).toContain("42");
  });

  it("reports integration degradation on L5", () => {
    const layers = deriveAgentFactoryLayers(
      createContext({
        integrations: [
          { id: "crc", name: "CRC Bridge", status: "degraded", lastEvent: ISO },
          { id: "ci", name: "CI", status: "healthy", lastEvent: ISO },
        ],
        uploadReceipts: [
          {
            id: "drop-1",
            workspaceId: "ws",
            dropId: "1",
            dropType: "crc",
            originalName: "artifact.zip",
            casKeys: ["cas://abc"],
            receiptPath: "path",
            uploadedAt: ISO,
            uploadedBy: { id: "owner", name: "Kai" },
          },
        ],
      }),
    );
    const l5 = layers.find((layer) => layer.id === "L5");
    expect(l5?.state).toBe("alert");
    expect(l5?.signalValue.toLowerCase()).toContain("drop");
    expect(l5?.metricValue).toContain("1/");
  });
});
