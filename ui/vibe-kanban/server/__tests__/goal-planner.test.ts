import { describe, expect, it, vi } from "vitest";

import {
  createWorkflowFromGoal,
  planGoal,
  slugifyStage,
  type GoalPayload,
} from "../goal-planner";

describe("goal-planner", () => {
  it("creates deterministic workflow definitions", () => {
    const goal: GoalPayload = {
      id: "goal-alpha",
      title: "Align release",
      summary: "Prepare the upcoming release",
      workspaceId: "studio",
      boardId: "launchpad",
      createdBy: "ava",
    };

    const workflow = createWorkflowFromGoal(goal);
    expect(workflow.name).toBe("goal-alpha");
    expect(workflow.version).toBe("2024.05");
    expect(workflow.stages).toHaveLength(3);
    expect(workflow.stages.map((stage) => stage.id)).toEqual([
      slugifyStage("goal-intake"),
      slugifyStage("context-sync"),
      slugifyStage("plan-execution"),
    ]);
    expect(workflow.stages[0].tasks[0].parameters.goalId).toBe("goal-alpha");
  });

  it("posts workflow payloads to the UI API", async () => {
    const goal: GoalPayload = {
      title: "Ship sprint",
      workspaceId: "studio",
      boardId: "launchpad",
      createdBy: "ava",
    };

    const fetchImpl = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        workflowId: "goal-123",
        state: "pending",
        resumeToken: {
          workflowId: "goal-123",
          stageId: "goal-intake",
          checkpoint: "stage://goal-123/goal-intake",
          issuedAt: "2024-06-01T00:00:00.000Z",
          expiresAt: "2024-06-01T04:00:00.000Z",
        },
        stages: [
          { id: "goal-intake", name: "Goal Intake", state: "pending" },
        ],
      }),
    });

    const result = await planGoal(goal, { fetchImpl, uiApiBaseUrl: "http://localhost:8787" });

    expect(fetchImpl).toHaveBeenCalledWith(
      "http://localhost:8787/ui/workflows",
      expect.objectContaining({ method: "POST" })
    );
    expect(result.workflowId).toBe("goal-123");
    expect(result.resumeToken?.workflowId).toBe("goal-123");
    expect(result.stages[0].id).toBe("goal-intake");
  });
});
