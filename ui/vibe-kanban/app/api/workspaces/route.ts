import { NextResponse } from "next/server";

import { assertUser } from "@/app/lib/session";
import { listWorkspacesForUser } from "@/server/workspace-store";
import { appendGoalTrace } from "@/server/memory-store";

export async function GET() {
  const user = assertUser();
  const workspaces = await listWorkspacesForUser(user.id);
  const now = new Date().toISOString();
  await Promise.all(
    workspaces.map((workspace) =>
      appendGoalTrace({
        id: `${workspace.id}-trace-${Date.now()}-${Math.random().toString(16).slice(2, 10)}`,
        goalId: `workspace:${workspace.id}`,
        workspaceId: workspace.id,
        boardId: undefined,
        actorId: user.id,
        actorName: user.name,
        action: "workspace.listed",
        summary: `Workspace listed for ${user.name ?? "unknown"}`,
        metadata: { endpoint: "/api/workspaces" },
        createdAt: now,
      })
    )
  );
  return NextResponse.json({ workspaces });
}
