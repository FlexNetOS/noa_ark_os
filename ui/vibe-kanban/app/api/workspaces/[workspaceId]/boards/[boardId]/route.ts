import { NextResponse } from "next/server";

import { assertUser } from "../../../../lib/session";
import { getBoard, getWorkspace, removeBoard, saveBoard } from "../../../../../server/workspace-store";
import { workspaceEventHub } from "../../../../../server/workspace-events";
import type { WorkspaceBoard } from "../../../../components/board-types";

export async function GET(
  _request: Request,
  { params }: { params: { workspaceId: string; boardId: string } }
) {
  const user = assertUser();
  const workspace = await getWorkspace(params.workspaceId);
  if (!workspace || !workspace.members.some((member) => member.id === user.id)) {
    return new NextResponse("Not Found", { status: 404 });
  }
  const board = workspace.boards.find((item) => item.id === params.boardId);
  if (!board) {
    return new NextResponse("Not Found", { status: 404 });
  }
  return NextResponse.json({ board });
}

export async function PUT(
  request: Request,
  { params }: { params: { workspaceId: string; boardId: string } }
) {
  const user = assertUser();
  const workspace = await getWorkspace(params.workspaceId);
  if (!workspace) {
    return new NextResponse("Not Found", { status: 404 });
  }
  const member = workspace.members.find((m) => m.id === user.id);
  if (!member) {
    return new NextResponse("Forbidden", { status: 403 });
  }
  const payload = (await request.json()) as WorkspaceBoard;
  const board = await getBoard(params.workspaceId, params.boardId);
  if (!board) {
    return new NextResponse("Not Found", { status: 404 });
  }
  const nextBoard: WorkspaceBoard = {
    ...board,
    ...payload,
    id: params.boardId,
    workspaceId: params.workspaceId,
    lastUpdated: new Date().toISOString(),
  };
  const { board: saved, activity } = await saveBoard(params.workspaceId, nextBoard, member);
  workspaceEventHub.publishBoardUpdate(params.workspaceId, params.boardId);
  workspaceEventHub.publishActivity(params.workspaceId, activity);
  workspaceEventHub.publishNotification(params.workspaceId, {
    id: `notif-${Date.now()}`,
    message: `${member.name} synced ${saved.projectName}`,
    createdAt: new Date().toISOString(),
    severity: "success",
  });
  return NextResponse.json({ board: saved });
}

export async function DELETE(
  _request: Request,
  { params }: { params: { workspaceId: string; boardId: string } }
) {
  const user = assertUser();
  const workspace = await getWorkspace(params.workspaceId);
  if (!workspace) {
    return new NextResponse("Not Found", { status: 404 });
  }
  const member = workspace.members.find((m) => m.id === user.id);
  if (!member) {
    return new NextResponse("Forbidden", { status: 403 });
  }
  await removeBoard(params.workspaceId, params.boardId, member);
  workspaceEventHub.publishNotification(params.workspaceId, {
    id: `notif-${Date.now()}`,
    message: `${member.name} archived a board`,
    createdAt: new Date().toISOString(),
    severity: "warning",
  });
  return NextResponse.json({ ok: true });
}
