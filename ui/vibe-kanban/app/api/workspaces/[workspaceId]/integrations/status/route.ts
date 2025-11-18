import { NextResponse } from "next/server";

import { assertUser } from "@/app/lib/session";
import { getWorkspace } from "@/server/workspace-store";

const demoIntegrations = [
  {
    id: "crc",
    name: "CRC Runtime",
    status: "healthy",
    lastEvent: "All sandboxes deployed 2 minutes ago",
  },
  {
    id: "cicd",
    name: "CI/CD",
    status: "running",
    lastEvent: "Ship pipeline triggered for Vibe Kanban",
  },
  {
    id: "agent-factory",
    name: "Agent Factory",
    status: "healthy",
    lastEvent: "Standing by for new missions",
  },
];

export async function GET(_request: Request, { params }: { params: { workspaceId: string } }) {
  const user = assertUser();
  const workspace = await getWorkspace(params.workspaceId);
  if (!workspace || !workspace.members.some((member) => member.id === user.id)) {
    return new NextResponse("Not Found", { status: 404 });
  }
  return NextResponse.json({ integrations: demoIntegrations });
}
