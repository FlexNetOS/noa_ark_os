/**
 * Next.js route handler that powers the Kanban AI assist button.
 * Delegates validation and template rendering to the shared server gateway.
 */

import { NextResponse } from "next/server";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

import { handlePromptRequest } from "@noa-ark/server/ai/controllers/prompt";
import { getProvider } from "@noa-ark/server/ai/router";

import { aiDatabase } from "@/server/ai-database";
import { aiRateLimiter } from "@/server/rate-limiter";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

let cachedTemplate: string | null = null;

async function loadTemplate() {
  if (cachedTemplate) {
    return cachedTemplate;
  }
  const templatePath = resolve(
    process.cwd(),
    "app/features/ai_assist/prompt_templates/feature_builder.md",
  );
  cachedTemplate = await readFile(templatePath, "utf8");
  return cachedTemplate;
}

function getClientIdentity(request: Request) {
  const forwarded = request.headers.get("x-forwarded-for");
  if (forwarded) {
    return forwarded.split(",")[0]?.trim() ?? "unknown";
  }
  const forwardedHost = request.headers.get("x-real-ip");
  if (forwardedHost) {
    return forwardedHost;
  }
  return "unknown";
}

export async function POST(request: Request) {
  const identity = getClientIdentity(request);
  if (!aiRateLimiter.consume(identity)) {
    console.warn(JSON.stringify({ type: "ai_assist.rate_limited", identity }));
    return NextResponse.json(
      { error: "Rate limit exceeded. Please wait a minute before trying again." },
      { status: 429 },
    );
  }

  let body: unknown;
  try {
    body = await request.json();
  } catch (error) {
    console.error(
      JSON.stringify({
        type: "ai_assist.invalid_json",
        error: error instanceof Error ? error.message : error,
      }),
    );
    return NextResponse.json({ error: "Invalid JSON body." }, { status: 400 });
  }

  const provider = getProvider();

  try {
    const result = await handlePromptRequest(body, {
      loadTemplate,
      provider,
      logRequest: async ({
        cardId,
        title,
        provider: providerName,
        status,
        latencyMs,
        errorMessage,
      }) => {
        aiDatabase.logRequest({
          source: "kanban",
          cardId,
          title,
          provider: providerName,
          latencyMs,
          status,
          errorMsg: errorMessage,
        });
      },
    });

    return NextResponse.json({
      prompt: result.prompt,
      provider: result.provider,
      completion: result.completion,
    });
  } catch (error) {
    console.error(
      JSON.stringify({
        type: "ai_assist.error",
        identity,
        error: error instanceof Error ? error.message : error,
      }),
    );
    return NextResponse.json({ error: "Failed to build AI prompt." }, { status: 500 });
  }
}
