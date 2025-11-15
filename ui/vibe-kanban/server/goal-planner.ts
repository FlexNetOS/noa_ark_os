import { randomUUID } from "crypto";

import type { ResumeToken } from "@noa-ark/shared-ui/schema";

export type GoalPayload = {
  id?: string;
  title: string;
  summary?: string;
  workspaceId: string;
  boardId: string;
  createdBy: string;
  focusCardId?: string;
  context?: Record<string, unknown>;
};

export type WorkflowTaskDefinition = {
  agent: string;
  action: string;
  parameters: Record<string, unknown>;
};

export type WorkflowStageDefinition = {
  id: string;
  name: string;
  stageType: "sequential" | "parallel" | "conditional" | "loop";
  dependsOn: string[];
  tasks: WorkflowTaskDefinition[];
};

export type WorkflowDefinition = {
  name: string;
  version: string;
  stages: WorkflowStageDefinition[];
};

export type PlannerOptions = {
  uiApiBaseUrl?: string;
  fetchImpl?: typeof fetch;
};

export type PlannerResult = {
  workflowId: string;
  workflow: WorkflowDefinition;
  stages: WorkflowStageDefinition[];
  resumeToken?: ResumeToken;
};

type WorkflowStartResponse = {
  workflowId: string;
  state: string;
  resumeToken?: ResumeToken;
  stages: { id: string; name: string; state: string }[];
};

const DEFAULT_UI_API = process.env.UI_API_URL ?? process.env.NEXT_PUBLIC_UI_API ?? "http://localhost:8787";

export function createWorkflowFromGoal(goal: GoalPayload): WorkflowDefinition {
  const workflowId = goal.id ?? `goal-${randomUUID().slice(0, 8)}`;
  const normalizedTitle = goal.title.trim() || "Workspace Goal";

  const stages: WorkflowStageDefinition[] = [
    {
      id: slugifyStage("goal-intake"),
      name: "Goal Intake",
      stageType: "sequential",
      dependsOn: [],
      tasks: [
        {
          agent: "goal.curator",
          action: "ingest",
          parameters: {
            goalId: workflowId,
            title: normalizedTitle,
            summary: goal.summary ?? "",
            createdBy: goal.createdBy,
            boardId: goal.boardId,
            workspaceId: goal.workspaceId,
          },
        },
      ],
    },
    {
      id: slugifyStage("context-sync"),
      name: "Context Sync",
      stageType: "parallel",
      dependsOn: [slugifyStage("goal-intake")],
      tasks: [
        {
          agent: "context.collector",
          action: "aggregate",
          parameters: {
            goalId: workflowId,
            focusCardId: goal.focusCardId ?? null,
            boardSnapshot: goal.context?.boardSnapshot ?? null,
          },
        },
        {
          agent: "signal.observer",
          action: "summarize",
          parameters: {
            goalId: workflowId,
            signals: goal.context?.signals ?? [],
          },
        },
      ],
    },
    {
      id: slugifyStage("plan-execution"),
      name: "Plan Execution",
      stageType: "sequential",
      dependsOn: [slugifyStage("context-sync")],
      tasks: [
        {
          agent: "automation.designer",
          action: "draft_workflow",
          parameters: {
            goalId: workflowId,
            workspaceId: goal.workspaceId,
            boardId: goal.boardId,
            focusCardId: goal.focusCardId ?? null,
            title: normalizedTitle,
          },
        },
        {
          agent: "automation.runner",
          action: "activate",
          parameters: {
            goalId: workflowId,
            workspaceId: goal.workspaceId,
            boardId: goal.boardId,
          },
        },
      ],
    },
  ];

  return {
    name: workflowId,
    version: "2024.05",
    stages,
  };
}

export async function planGoal(goal: GoalPayload, options: PlannerOptions = {}): Promise<PlannerResult> {
  const workflow = createWorkflowFromGoal(goal);
  const fetchImpl = options.fetchImpl ?? fetch;
  const baseUrl = (options.uiApiBaseUrl ?? DEFAULT_UI_API).replace(/\/$/, "");

  const response = await fetchImpl(`${baseUrl}/ui/workflows`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ workflow }),
  });

  if (!response.ok) {
    const errorBody = await response.text().catch(() => "unknown");
    throw new Error(`Planner failed to start workflow (${response.status}): ${errorBody}`);
  }

  const payload = (await response.json()) as WorkflowStartResponse;

  const stageMap = new Map(workflow.stages.map((stage) => [stage.id, stage] as const));
  const mergedStages = payload.stages.map((stage) => {
    const definition = stageMap.get(stage.id);
    return (
      definition ?? {
        id: stage.id,
        name: stage.name,
        stageType: "sequential",
        dependsOn: [],
        tasks: [],
      }
    );
  });

  return {
    workflowId: payload.workflowId,
    workflow,
    stages: mergedStages,
    resumeToken: payload.resumeToken,
  };
}

export function slugifyStage(input: string): string {
  return input
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}
