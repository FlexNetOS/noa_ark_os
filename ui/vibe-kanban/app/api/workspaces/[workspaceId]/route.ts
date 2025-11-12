import { NextResponse } from "next/server";

import { assertUser } from "../../../lib/session";
import { getWorkspace, upsertWorkspace } from "../../../../server/workspace-store";

export async function GET(
  _request: Request,
  { params }: { params: { workspaceId: string } }
) {
  const user = assertUser();
  const workspace = await getWorkspace(params.workspaceId);
  if (!workspace || !workspace.members.some((member) => member.id === user.id)) {
    return new NextResponse("Not Found", { status: 404 });
  }
  return NextResponse.json({ workspace });
}

export async function PATCH(
  request: Request,
  { params }: { params: { workspaceId: string } }
) {
  const user = assertUser();
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
  return NextResponse.json({ workspace: updated });
}
