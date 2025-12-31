import { z } from "zod";

import type { AIProvider } from "../types";

const PromptPayloadSchema = z
  .object({
    cardId: z.string().max(128).optional(),
    title: z.string().min(1).max(512),
    description: z.string().max(4000).optional(),
    labels: z.array(z.string().min(1).max(128)).max(16).optional(),
    checklist: z.array(z.string().min(1).max(512)).max(32).optional(),
    contextPaths: z.array(z.string().min(1).max(256)).max(16).optional(),
  })
  .strict();

export type PromptPayload = z.infer<typeof PromptPayloadSchema>;

export interface PromptControllerDependencies {
  loadTemplate(): Promise<string>;
  provider?: AIProvider | null;
  logRequest(entry: AiRequestLogEntry): Promise<void>;
  now?: () => number;
}

export type AiRequestLogEntry = {
  cardId?: string;
  title: string;
  provider: string | null;
  status: "success" | "error";
  latencyMs: number;
  errorMessage?: string;
};

export type PromptControllerResult = {
  prompt: string;
  provider?: string | null;
  completion?: string | null;
};

export function validatePromptPayload(input: unknown): PromptPayload {
  return PromptPayloadSchema.parse(input);
}

function toSlug(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 64);
}

function renderTemplate(template: string, payload: PromptPayload) {
  const labelsCsv = payload.labels?.join(", ") ?? "none";
  const descriptionLines: string[] = [];
  if (payload.description && payload.description.trim().length > 0) {
    descriptionLines.push(payload.description.trim());
  } else {
    descriptionLines.push("No additional description provided.");
  }
  if (payload.contextPaths && payload.contextPaths.length > 0) {
    descriptionLines.push("Suggested context paths:");
    for (const path of payload.contextPaths) {
      descriptionLines.push(`- ${path}`);
    }
  }

  const checklistBullets =
    payload.checklist && payload.checklist.length > 0
      ? payload.checklist.map((item) => `- ${item}`).join("\n")
      : "- (none provided)";

  return template
    .replace(/{{title}}/g, payload.title)
    .replace(/{{labels_csv}}/g, labelsCsv)
    .replace(/{{description_or_default}}/g, descriptionLines.join("\n"))
    .replace(/{{checklist_bullets}}/g, checklistBullets)
    .replace(/{{slug}}/g, toSlug(payload.title));
}

export async function handlePromptRequest(
  input: unknown,
  deps: PromptControllerDependencies,
): Promise<PromptControllerResult> {
  const payload = validatePromptPayload(input);
  const template = await deps.loadTemplate();
  const now = deps.now ?? (() => Date.now());
  const started = now();

  try {
    const prompt = renderTemplate(template, payload);
    let completion: string | null = null;
    const providerName = deps.provider?.name ?? null;

    if (deps.provider && deps.provider.isConfigured()) {
      completion = await deps.provider.completePrompt(prompt);
    }

    const finished = now();
    await deps.logRequest({
      cardId: payload.cardId,
      title: payload.title,
      provider: providerName,
      status: "success",
      latencyMs: finished - started,
    });

    return {
      prompt,
      provider: providerName,
      completion,
    };
  } catch (error) {
    const finished = now();
    const message = error instanceof Error ? error.message : "Unknown error";
    await deps.logRequest({
      cardId: payload.cardId,
      title: payload.title,
      provider: deps.provider?.name ?? null,
      status: "error",
      latencyMs: finished - started,
      errorMessage: message,
    });
    throw error;
  }
}

export { renderTemplate as renderPromptTemplate };
