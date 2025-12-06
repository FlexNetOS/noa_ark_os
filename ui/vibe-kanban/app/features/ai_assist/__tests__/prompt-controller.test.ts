import { describe, expect, it, vi } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

import { handlePromptRequest } from "server/ai/controllers/prompt";

class MockProvider {
  readonly name = "test-provider";
  isConfigured() {
    return true;
  }
  async completePrompt(prompt: string) {
    return `completed:${prompt.length}`;
  }
}

describe("handlePromptRequest", () => {
  it("renders the template and records success", async () => {
    const currentDir = dirname(fileURLToPath(import.meta.url));
    const templatePath = resolve(currentDir, "../prompt_templates/feature_builder.md");
    const template = readFileSync(templatePath, "utf8");
    const logRequest = vi.fn();
    const result = await handlePromptRequest(
      {
        goalId: "goal-1",
        title: "Sync gateway telemetry",
        description: "Collect logs from gateway service",
        labels: ["gateway", "observability"],
        checklist: ["Check dashboard", "Update docs"],
        contextPaths: ["server/gateway", "docs/observability"],
      },
      {
        loadTemplate: async () => template,
        provider: new MockProvider(),
        logRequest,
      }
    );

    expect(result.prompt).toContain("Feature: Sync gateway telemetry");
    expect(result.prompt).not.toContain("{{");
    expect(result.provider).toBe("test-provider");
    expect(result.completion).toBe("completed:" + result.prompt.length);
    expect(logRequest).toHaveBeenCalledWith(
      expect.objectContaining({ status: "success", provider: "test-provider" })
    );
  });
});
