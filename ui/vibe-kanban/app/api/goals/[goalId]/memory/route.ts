import { NextResponse } from "next/server";

import { assertUser } from "@/app/lib/session";
import { aiDatabase } from "@/server/ai-database";
import { appendGoalTrace, getGoalMemoryInsights } from "@/server/memory-store";
import { getWorkspace } from "@/server/workspace-store";

export async function GET(request: Request, { params }: { params: { goalId: string } }) {
  const user = assertUser();
  const url = new URL(request.url);
  const workspaceId = url.searchParams.get("workspaceId");
  if (!workspaceId) {
    return new NextResponse("workspaceId query parameter required", { status: 400 });
  }
  const workspace = await getWorkspace(workspaceId);
  if (!workspace || !workspace.members.some((member) => member.id === user.id)) {
    return new NextResponse("Not Found", { status: 404 });
  }
  const canonicalGoalId = params.goalId.includes(":") ? params.goalId : `board:${params.goalId}`;
  const memory = await getGoalMemoryInsights(canonicalGoalId, workspaceId);
  const lifecycle = aiDatabase.listGoalLifecycleEvents(canonicalGoalId, 50);
  const artifacts = aiDatabase.listArtifacts(canonicalGoalId, 25);
  const similarGoals = aiDatabase.searchSimilarGoals(canonicalGoalId, "kanban.board.stats.v1", 5);
  aiDatabase.recordGoalLifecycleEvent({
    goalId: canonicalGoalId,
    workspaceId,
    eventType: "memory.requested",
    status: "success",
    summary: `Memory retrieved by ${user.name ?? user.id}`,
    payload: { goalId: canonicalGoalId, workspaceId },
  });
  await appendGoalTrace({
    id: `${canonicalGoalId}-memory-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`,
    goalId: canonicalGoalId,
    workspaceId,
    boardId: canonicalGoalId.startsWith("board:")
      ? canonicalGoalId.replace("board:", "")
      : undefined,
    actorId: user.id,
    actorName: user.name,
    action: "memory.requested",
    summary: "Planner memory retrieved",
    metadata: { endpoint: request.url },
    createdAt: new Date().toISOString(),
  });
  const insightSummary = buildInsightSummary(
    memory.traceCount,
    lifecycle.length,
    artifacts.length,
    similarGoals.length,
  );
  return NextResponse.json({
    goalId: canonicalGoalId,
    workspaceId,
    summary: memory.summary,
    traceCount: memory.traceCount,
    lastSeen: memory.lastSeen,
    traces: memory.traces,
    lifecycle,
    artifacts,
    similarGoals,
    insightSummary,
  });
}

function buildInsightSummary(
  traceCount: number,
  lifecycleEvents: number,
  artifactCount: number,
  similarGoals: number,
) {
  return [
    traceCount ? `${traceCount} execution traces` : "no execution traces",
    lifecycleEvents ? `${lifecycleEvents} lifecycle events` : "no lifecycle events yet",
    artifactCount ? `${artifactCount} artifacts` : "no artifacts captured",
    similarGoals ? `${similarGoals} similar goals indexed` : "no similar goals indexed",
  ].join(", ");
}
