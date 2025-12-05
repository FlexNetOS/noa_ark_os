import { promises as fs } from "node:fs";
import { Readable } from "node:stream";
import { mkdtempSync } from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";

import { POST } from "../route";

const mockWorkspace = {
  id: "studio",
  name: "Studio",
  accent: "from-indigo-500 via-purple-500 to-blue-500",
  createdAt: new Date().toISOString(),
  billingPlan: "starter" as const,
  members: [
    { id: "user-1", name: "Ava", role: "owner", avatarHue: 265 },
  ],
  boards: [],
  activity: [],
  notifications: [],
  uploadReceipts: [],
};

vi.mock("@/app/lib/session", () => ({
  assertUser: async () => ({ id: "user-1", name: "Ava" }),
}));

const getWorkspace = vi.fn();
const recordUploadReceipt = vi.fn();
const recordWorkspaceNotification = vi.fn();
const publishNotification = vi.fn();

vi.mock("@/server/workspace-store", () => ({
  getWorkspace: (...args: unknown[]) => getWorkspace(...args),
  recordUploadReceipt: (...args: unknown[]) => recordUploadReceipt(...args),
  recordWorkspaceNotification: (...args: unknown[]) => recordWorkspaceNotification(...args),
}));

vi.mock("@/server/workspace-events", () => ({
  workspaceEventHub: { publishNotification: (...args: unknown[]) => publishNotification(...args) },
}));

describe("POST /api/uploads", () => {
  const originalFetch = global.fetch;
  let tmpDir: string;

  beforeEach(() => {
    tmpDir = mkdtempSync(path.join(tmpdir(), "uploads-test-"));
    process.env.UPLOAD_TMP = tmpDir;
    getWorkspace.mockResolvedValue({ ...mockWorkspace });
    vi.spyOn(fs, "readFile").mockResolvedValue(Buffer.from("hello"));
    const uploadedAt = new Date().toISOString();
    recordUploadReceipt.mockResolvedValue({
      id: "upload-1",
      workspaceId: "studio",
      dropId: "drop-1",
      dropType: "repos",
      originalName: "upload.tar.gz",
      casKeys: ["hash-1"],
      receiptPath: "/tmp/receipt.json",
      uploadedAt,
      uploadedBy: { id: "user-1", name: "Ava" },
    });
    recordWorkspaceNotification.mockResolvedValue({
      id: "notif-1",
      message: "Ava uploaded upload.tar.gz",
      createdAt: uploadedAt,
      severity: "success",
      casKeys: ["hash-1"],
      receiptPath: "/tmp/receipt.json",
      href: "file:///tmp/receipt.json",
    });
    global.fetch = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          drop_id: "drop-1",
          status: "incoming",
          cas_keys: ["hash-1"],
          receipt_path: "/tmp/receipt.json",
        }),
        { status: 200, headers: { "Content-Type": "application/json" } }
      )
    );
  });

  afterEach(async () => {
    vi.restoreAllMocks();
    global.fetch = originalFetch;
    delete process.env.UPLOAD_TMP;
    await fs.rm(tmpDir, { recursive: true, force: true });
  });

  it("forwards uploads to the bridge and records receipts", async () => {
    const formData = new FormData();
    formData.append("workspaceId", "studio");
    formData.append("dropType", "repos");
    formData.append("boardId", "launchpad");
    const blob = new File(["hello"], "upload.tar.gz", { type: "application/gzip" });
    Object.defineProperty(blob, "stream", {
      value: () => Readable.toWeb(Readable.from(Buffer.from("hello"))),
    });
    formData.append("file", blob);

    const request = {
      formData: async () => formData,
    } as unknown as Request;

    const response = await POST(request);
    expect(response.status).toBe(200);

    expect(global.fetch).toHaveBeenCalledWith(
      expect.stringMatching(/\/ui\/drop-in\/upload$/),
      expect.objectContaining({ method: "POST" })
    );

    expect(recordUploadReceipt).toHaveBeenCalledWith(
      "studio",
      expect.objectContaining({ dropId: "drop-1", originalName: "upload.tar.gz" })
    );
    expect(recordWorkspaceNotification).toHaveBeenCalled();
    expect(publishNotification).toHaveBeenCalled();

    const payload = await response.json();
    expect(payload.casKeys).toEqual(["hash-1"]);
    expect(payload.upload.receiptPath).toBe("/tmp/receipt.json");
    expect(payload.notification.href).toContain("/tmp/receipt.json");
  }, 15000);
});
