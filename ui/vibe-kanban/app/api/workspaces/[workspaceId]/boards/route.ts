import { NextResponse } from "next/server";

import { assertUser } from "../../../../lib/session";
import { createBoard, getWorkspace } from "../../../../../server/workspace-store";
import { workspaceEventHub } from "../../../../../server/workspace-events";
import type { WorkspaceBoard } from "../../../../components/board-types";
import { assertUser } from "@/app/lib/session";
import { createBoard, getWorkspace } from "@/server/workspace-store";
import { workspaceEventHub } from "@/server/workspace-events";
import type { WorkspaceBoard } from "@/app/components/board-types";

export async function GET(
  _request: Request,
  { params }: { params: { workspaceId: string } }
) {
  const user = assertUser();
  const workspace = await getWorkspace(params.workspaceId);
  if (!workspace || !workspace.members.some((member) => member.id === user.id)) {
    return new NextResponse("Not Found", { status: 404 });
  }
  return NextResponse.json({ boards: workspace.boards });
}

export async function POST(
  request: Request,
  { params }: { params: { workspaceId: string } }
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
  const payload = (await request.json()) as Partial<WorkspaceBoard>;
  const now = new Date().toISOString();
  const board: Omit<WorkspaceBoard, "workspaceId" | "metrics"> = {
    id: payload.id ?? `board-${Date.now()}`,
    projectName: payload.projectName?.trim() || "Untitled Board",
    description: payload.description ?? "",
    lastUpdated: now,
    accent: payload.accent ?? "from-indigo-500 via-purple-500 to-blue-500",
    columns:
      payload.columns?.length
        ? payload.columns
        : [
            { id: "todo", title: "To Do", accent: payload.accent ?? "from-indigo-500 via-purple-500 to-blue-500", cards: [] },
            { id: "in-progress", title: "In Progress", accent: "from-sky-500 via-cyan-400 to-emerald-400", cards: [] },
            { id: "done", title: "Completed", accent: "from-violet-500 via-indigo-400 to-fuchsia-500", cards: [] },
          ],
    archived: false,
    moodSamples: payload.moodSamples ?? [],
  };
  const { board: created, activity } = await createBoard(params.workspaceId, board, member);
  workspaceEventHub.publishActivity(params.workspaceId, activity);
  workspaceEventHub.publishNotification(params.workspaceId, {
    id: `notif-${Date.now()}`,
    message: `${member.name} created ${created.projectName}`,
    createdAt: new Date().toISOString(),
    severity: "success",
  });
  return NextResponse.json({ board: created, activity }, { status: 201 });
}
