import { NextResponse } from "next/server";

import { assertUser } from "@/app/lib/session";
import { listWorkspacesForUser } from "@/server/workspace-store";
import { appendGoalTrace } from "@/server/memory-store";

export async function GET() {
  const user = assertUser();
  const workspaces = await listWorkspacesForUser(user.id);
  const now = new Date().toISOString();
  await appendGoalTrace({
    id: `workspaces-list-trace-${Date.now()}-${Math.random().toString(16).slice(2, 10)}`,
    goalId: `workspaces:list`,
    workspaceId: undefined,
    boardId: undefined,
    actorId: user.id,
    actorName: user.name,
    action: "workspaces.listed",
    summary: `User ${user.name ?? "unknown"} listed their workspaces (${workspaces.length} total)`,
    metadata: { endpoint: "/api/workspaces", workspaceCount: workspaces.length },
    createdAt: now,
  });
  return NextResponse.json({ workspaces });
}
