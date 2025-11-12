import { NextResponse } from "next/server";

import { assertUser } from "@/app/lib/session";
import { listWorkspacesForUser } from "../../../server/workspace-store";
import { listWorkspacesForUser } from "@/server/workspace-store";

export async function GET() {
  const user = assertUser();
  const workspaces = await listWorkspacesForUser(user.id);
  return NextResponse.json({ workspaces });
}
