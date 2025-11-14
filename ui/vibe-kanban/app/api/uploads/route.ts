import { NextResponse } from "next/server";
import { createWriteStream } from "node:fs";
import { promises as fs } from "node:fs";
import path from "node:path";
import { pipeline } from "node:stream/promises";
import { Readable } from "node:stream";
import { ReadableStream as WebReadableStream } from "node:stream/web";

import { assertUser } from "@/app/lib/session";
import { getWorkspace, recordUploadReceipt, recordWorkspaceNotification } from "@/server/workspace-store";
import { workspaceEventHub } from "@/server/workspace-events";
import { ensureTraceId, logError, logInfo } from "@noa-ark/shared-ui/logging";

export const runtime = "nodejs";

function resolveUploadDir() {
  return process.env.UPLOAD_TMP ? path.resolve(process.env.UPLOAD_TMP) : path.join(process.cwd(), "tmp/uploads");
}

function sanitizeFilename(name: string | undefined) {
  if (!name) return "upload.bin";
  return name.replace(/[^a-zA-Z0-9._-]/g, "_");
}

export async function POST(request: Request) {
  const user = assertUser();
  const traceSource = typeof request.headers?.get === "function" ? request.headers.get("x-trace-id") : null;
  const traceId = ensureTraceId(traceSource);
  const component = "api.uploads";

  let formData: FormData;
  try {
    formData = await request.formData();
  } catch (error) {
    logError({
      component,
      event: "invalid_form_data",
      message: "Uploads API rejected malformed form data",
      outcome: "rejected",
      traceId,
      context: {},
      error,
    });
    return NextResponse.json({ error: "Invalid form data payload." }, { status: 400 });
  }

  const workspaceId = String(formData.get("workspaceId") ?? "").trim();
  const dropType = String(formData.get("dropType") ?? formData.get("type") ?? "").trim();
  const boardId = String(formData.get("boardId") ?? "").trim() || undefined;
  const file = formData.get("file");

  if (!workspaceId) {
    return NextResponse.json({ error: "workspaceId is required." }, { status: 400 });
  }
  if (!dropType) {
    return NextResponse.json({ error: "dropType is required." }, { status: 400 });
  }
  if (!(file instanceof File) || file.size === 0) {
    return NextResponse.json({ error: "file is required." }, { status: 400 });
  }

  const workspace = await getWorkspace(workspaceId);
  if (!workspace) {
    return NextResponse.json({ error: "Workspace not found." }, { status: 404 });
  }
  if (!workspace.members.some((member) => member.id === user.id)) {
    return NextResponse.json({ error: "Forbidden." }, { status: 403 });
  }

  const uploadDir = resolveUploadDir();
  await fs.mkdir(uploadDir, { recursive: true });
  const safeName = sanitizeFilename(file.name);
  const tempPath = path.join(uploadDir, `${Date.now()}-${safeName}`);

  try {
    const readable = Readable.fromWeb(file.stream() as unknown as WebReadableStream);
    await pipeline(readable, createWriteStream(tempPath));
  } catch (error) {
    logError({
      component,
      event: "persist_failed",
      message: "Failed to persist incoming upload",
      outcome: "failure",
      traceId,
      context: { workspaceId, dropType },
      error,
    });
    await fs.rm(tempPath, { force: true }).catch(() => undefined);
    return NextResponse.json({ error: "Failed to persist upload." }, { status: 500 });
  }

  const bridgeBase = process.env.UPLOAD_BRIDGE_URL ?? process.env.NEXT_PUBLIC_UI_API ?? "http://localhost:8787";
  const endpoint = new URL("/ui/drop-in/upload", bridgeBase).toString();

  const forwardForm = new FormData();
  try {
    const buffer = await fs.readFile(tempPath);
    forwardForm.append("file", new Blob([buffer]), safeName);
  } catch (error) {
    logError({
      component,
      event: "read_failed",
      message: "Failed to read temporary upload before forwarding",
      outcome: "failure",
      traceId,
      context: { workspaceId, dropType },
      error,
    });
    await fs.rm(tempPath, { force: true }).catch(() => undefined);
    return NextResponse.json({ error: "Unable to read temporary upload." }, { status: 500 });
  }
  forwardForm.append("drop_type", dropType);
  forwardForm.append("type", dropType);

  let bridgeResponse: Response;
  try {
    bridgeResponse = await fetch(endpoint, { method: "POST", body: forwardForm });
  } catch (error) {
    logError({
      component,
      event: "bridge_unreachable",
      message: "Upload bridge is unreachable",
      outcome: "failure",
      traceId,
      context: { endpoint, workspaceId, dropType },
      error,
    });
    await fs.rm(tempPath, { force: true }).catch(() => undefined);
    return NextResponse.json({ error: "Upload bridge unavailable." }, { status: 502 });
  }

  await fs.rm(tempPath, { force: true }).catch(() => undefined);

  if (!bridgeResponse.ok) {
    const errorBody = await bridgeResponse.text();
    logError({
      component,
      event: "bridge_error",
      message: "Upload bridge rejected the request",
      outcome: "failure",
      traceId,
      context: { endpoint, workspaceId, dropType, status: bridgeResponse.status, errorBody },
    });
    return NextResponse.json({ error: "Bridge rejected upload." }, { status: bridgeResponse.status });
  }

  let payload: {
    drop_id: string;
    status: string;
    cas_keys?: string[];
    receipt_path?: string;
    receipt_url?: string;
  };
  try {
    payload = (await bridgeResponse.json()) as typeof payload;
  } catch (error) {
    logError({
      component,
      event: "bridge_invalid_json",
      message: "Upload bridge returned invalid JSON",
      outcome: "failure",
      traceId,
      context: { endpoint, workspaceId, dropType },
      error,
    });
    return NextResponse.json({ error: "Invalid response from upload bridge." }, { status: 502 });
  }

  const casKeys = payload.cas_keys ?? [];
  const receiptPath = payload.receipt_path ?? "";
  const receiptUrl = payload.receipt_url ?? receiptPath;
  const uploadedAt = new Date().toISOString();

  const storedReceipt = await recordUploadReceipt(workspaceId, {
    boardId,
    dropId: payload.drop_id,
    dropType,
    originalName: safeName,
    casKeys,
    receiptPath,
    uploadedAt,
    uploadedBy: { id: user.id, name: user.name },
  });

  const notification = await recordWorkspaceNotification(workspaceId, {
    id: `notif-${Date.now()}`,
    message: `${user.name} uploaded ${safeName}`,
    createdAt: uploadedAt,
    severity: "success",
    href: receiptUrl,
    casKeys,
    receiptPath,
  });

  workspaceEventHub.publishNotification(workspaceId, notification);

  logInfo({
    component,
    event: "upload_forwarded",
    message: "Upload successfully forwarded to bridge",
    outcome: payload.status ?? "success",
    traceId,
    context: {
      workspaceId,
      dropType,
      dropId: payload.drop_id,
      boardId,
      casKeys,
    },
  });

  return NextResponse.json({
    dropId: payload.drop_id,
    status: payload.status,
    casKeys,
    receiptPath,
    receiptUrl,
    upload: storedReceipt,
    notification,
  });
}
