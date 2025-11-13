import { afterEach, describe, expect, it, vi } from "vitest";

import type { CapabilityRegistry } from "@/shared/capabilities";

describe("GET /api/capabilities", () => {
  afterEach(async () => {
    const module = await import("@/server/capabilities");
    if (typeof module.__resetCapabilityRegistryCacheForTests === "function") {
      module.__resetCapabilityRegistryCacheForTests();
    }
    vi.resetModules();
    vi.clearAllMocks();
    vi.unmock("fs/promises");
  });

  it("returns the capability registry with offline-friendly caching headers", async () => {
    const { GET } = await import("./route");
    const response = await GET();
    expect(response.status).toBe(200);
    const payload = (await response.json()) as CapabilityRegistry;
    expect(Array.isArray(payload.capabilities)).toBe(true);
    expect(response.headers.get("Cache-Control")).toContain("stale-while-revalidate");
    expect(response.headers.get("Cache-Control")).toContain("stale-if-error");
    expect(response.headers.get("CDN-Cache-Control")).toBeTruthy();
    expect(response.headers.get("Vercel-CDN-Cache-Control")).toBeTruthy();
  });

  it("falls back to an empty registry when the data file is missing", async () => {
    vi.resetModules();
    const readFile = vi.fn().mockRejectedValue(Object.assign(new Error("missing"), { code: "ENOENT" }));
    vi.doMock("fs/promises", () => ({
      default: { readFile },
      readFile,
    }));
    const { GET } = await import("./route");
    const response = await GET();
    const payload = (await response.json()) as CapabilityRegistry;
    expect(payload.capabilities).toEqual([]);
    expect(payload.version).toBe(1);
  });
});
