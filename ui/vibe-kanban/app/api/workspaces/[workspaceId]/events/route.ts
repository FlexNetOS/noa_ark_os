import { NextResponse } from "next/server";

import { assertUser } from "@/app/lib/session";
import { getWorkspace } from "@/server/workspace-store";
import { workspaceEventHub } from "@/server/workspace-events";

export const runtime = "nodejs";

export async function GET(_request: Request, { params }: { params: { workspaceId: string } }) {
  const user = assertUser();
  const workspace = await getWorkspace(params.workspaceId);
  if (!workspace || !workspace.members.some((member) => member.id === user.id)) {
    return new NextResponse("Not Found", { status: 404 });
  }

  const stream = new ReadableStream({
    start(controller) {
      const send = (event: string, data: unknown) => {
        controller.enqueue(`event: ${event}\n`);
        controller.enqueue(`data: ${JSON.stringify(data)}\n\n`);
      };

      const unsubscribe = workspaceEventHub.addListener((payload) => {
        if (payload.workspaceId !== params.workspaceId) return;
        send(payload.type, payload.data);
      });

      send("connected", { ok: true });
      send("presence", workspaceEventHub.snapshotPresence(params.workspaceId));

      return () => {
        unsubscribe();
      };
    },
    cancel() {
      // no-op for now
    },
  });

  return new Response(stream, {
    status: 200,
    headers: {
      "Cache-Control": "no-cache, no-transform",
      Connection: "keep-alive",
      "Content-Type": "text/event-stream",
    },
  });
}
