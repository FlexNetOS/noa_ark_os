import { NextResponse } from "next/server";

import { assertUser } from "@/app/lib/session";
import { appendGoalTrace } from "@/server/memory-store";
import { getWorkspace, upsertWorkspace } from "@/server/workspace-store";

export async function GET(
  _request: Request,
  { params }: { params: { workspaceId: string } }
) {
  const user = await assertUser();
  const workspace = await getWorkspace(params.workspaceId);
  if (!workspace || !workspace.members.some((member) => member.id === user.id)) {
    return new NextResponse("Not Found", { status: 404 });
  }
  await appendGoalTrace({
    id: `${params.workspaceId}-view-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`,
    goalId: `workspace:${params.workspaceId}`,
    workspaceId: params.workspaceId,
    boardId: undefined,
    actorId: user.id,
    actorName: user.name,
    action: "workspace.viewed",
    summary: `Workspace ${workspace.name} viewed`,
    metadata: { endpoint: `/api/workspaces/${params.workspaceId}` },
    createdAt: new Date().toISOString(),
  });
  return NextResponse.json({ workspace });
}

export async function PATCH(
  request: Request,
  { params }: { params: { workspaceId: string } }
) {
  const user = await assertUser();
  const workspace = await getWorkspace(params.workspaceId);
  if (!workspace || !workspace.members.some((member) => member.id === user.id)) {
    return new NextResponse("Not Found", { status: 404 });
  }
  const payload = await request.json();
  const updated = await upsertWorkspace({
    ...workspace,
    name: payload.name ? String(payload.name).trim() || workspace.name : workspace.name,
    billingPlan: payload.billingPlan ?? workspace.billingPlan,
  });
  await appendGoalTrace({
    id: `${params.workspaceId}-update-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`,
    goalId: `workspace:${params.workspaceId}`,
    workspaceId: params.workspaceId,
    boardId: undefined,
    actorId: user.id,
    actorName: user.name,
    action: "workspace.updated",
    summary: `Workspace ${workspace.name} updated`,
    metadata: { endpoint: `/api/workspaces/${params.workspaceId}`, changes: payload },
    createdAt: new Date().toISOString(),
  });
  return NextResponse.json({ workspace: updated });
}
