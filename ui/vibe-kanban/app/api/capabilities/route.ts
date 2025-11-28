import { NextResponse } from "next/server";

import { getCapabilityRegistry } from "@/server/capabilities";

export const revalidate = 60;

const CACHE_CONTROL = "public, max-age=60, stale-while-revalidate=86400, stale-if-error=604800";

export async function GET(): Promise<NextResponse> {
  const registry = await getCapabilityRegistry();
  return NextResponse.json(registry, {
    headers: {
      "Cache-Control": CACHE_CONTROL,
      "CDN-Cache-Control": CACHE_CONTROL,
      "Vercel-CDN-Cache-Control": CACHE_CONTROL,
    },
  });
}
