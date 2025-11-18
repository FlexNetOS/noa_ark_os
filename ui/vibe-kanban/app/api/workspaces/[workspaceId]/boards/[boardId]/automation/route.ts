import { NextResponse } from "next/server";

import { recordGoalAutomationProgress } from "@/server/workspace-store";
import type { ToolExecutionTelemetry } from "@/app/components/board-types";

function normaliseToolResults(input: unknown): ToolExecutionTelemetry[] {
  if (!Array.isArray(input)) return [];
  return input
    .map((entry) => ({
      name: typeof entry?.name === "string" ? entry.name : "tool",
      capability: typeof entry?.capability === "string" ? entry.capability : "unknown",
      status:
        entry?.status === "running" ||
        entry?.status === "failed" ||
        entry?.status === "succeeded" ||
        entry?.status === "pending" ||
        entry?.status === "skipped"
          ? entry.status
          : "pending",
      output: typeof entry?.output === "string" ? entry.output : undefined,
      error: typeof entry?.error === "string" ? entry.error : undefined,
      occurredAt: typeof entry?.occurredAt === "string" ? entry.occurredAt : undefined,
    }))
    .slice(0, 10);
}

export async function POST(
  request: Request,
  { params }: { params: { workspaceId: string; boardId: string } },
) {
  const body = await request.json().catch(() => ({}));
  const cardId = typeof body.cardId === "string" ? body.cardId : undefined;
  if (!cardId) {
    return NextResponse.json({ error: "cardId required" }, { status: 400 });
  }

  const status =
    body.status === "running" ||
    body.status === "completed" ||
    body.status === "failed" ||
    body.status === "queued"
      ? body.status
      : body.action === "retry"
        ? "queued"
        : "running";

  const agentId = typeof body.agentId === "string" ? body.agentId : "workflow.engine";
  const agentName = typeof body.agentName === "string" ? body.agentName : "Workflow Engine";
  const notes = typeof body.notes === "string" ? body.notes : undefined;
  const toolResults = normaliseToolResults(body.toolResults);

  try {
    const { automation, activity } = await recordGoalAutomationProgress(
      params.workspaceId,
      params.boardId,
      cardId,
      {
        agentId,
        agentName,
        status,
        notes,
        toolResults,
        attempt: typeof body.attempt === "number" ? body.attempt : undefined,
        occurredAt: typeof body.occurredAt === "string" ? body.occurredAt : undefined,
      },
    );
    return NextResponse.json({ automation, activity });
  } catch (error) {
    return NextResponse.json({ error: (error as Error).message }, { status: 400 });
  }
}
