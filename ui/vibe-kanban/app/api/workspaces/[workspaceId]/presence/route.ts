import { NextResponse } from "next/server";

import { assertUser } from "../../../../lib/session";
import { getWorkspace } from "../../../../../server/workspace-store";
import { workspaceEventHub } from "../../../../../server/workspace-events";
import { assertUser } from "@/app/lib/session";
import { getWorkspace } from "@/server/workspace-store";
import { workspaceEventHub } from "@/server/workspace-events";

export async function POST(
  request: Request,
  { params }: { params: { workspaceId: string } }
) {
  const user = assertUser();
  const workspace = await getWorkspace(params.workspaceId);
  if (!workspace || !workspace.members.some((member) => member.id === user.id)) {
    return new NextResponse("Not Found", { status: 404 });
  }
  const payload = await request.json();
  workspaceEventHub.heartbeat(
    params.workspaceId,
    typeof payload.boardId === "string" ? payload.boardId : undefined,
    user.id,
    user.name
  );
  return NextResponse.json({ ok: true });
}

export async function DELETE(
  _request: Request,
  { params }: { params: { workspaceId: string } }
) {
  const user = assertUser();
  const workspace = await getWorkspace(params.workspaceId);
  if (!workspace || !workspace.members.some((member) => member.id === user.id)) {
    return new NextResponse("Not Found", { status: 404 });
  }
  workspaceEventHub.removePresence(params.workspaceId, user.id);
  return NextResponse.json({ ok: true });
}
