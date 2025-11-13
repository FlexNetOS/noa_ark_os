import { describe, expect, it, vi } from "vitest";

import { getProvider as uiGetProvider, resetProviderCache as uiReset } from "../router";
import { getProvider as canonicalGetProvider, resetProviderCache as canonicalReset } from "@noa-ark/server/ai/router";
import { handlePromptRequest } from "../controllers/prompt";
import { LlamaCppProvider } from "@noa-ark/server/ai/providers/llama_cpp";

describe("UI gateway delegation", () => {
  it("re-exports the canonical router", () => {
    expect(uiGetProvider).toBe(canonicalGetProvider);
    expect(uiReset).toBe(canonicalReset);
  });

  it("invokes the llama.cpp engine through the shared prompt controller", async () => {
    const env = { LLAMA_CPP_ENDPOINT: "http://127.0.0.1:8042/v1" } as NodeJS.ProcessEnv;
    const fetchSpy = vi.fn(async () => ({
      ok: true,
      status: 200,
      json: async () => ({ choices: [{ text: "Rust engine: completion" }] }),
    })) as unknown as typeof fetch;

    const provider = new LlamaCppProvider(env, { fetchImpl: fetchSpy });

    const logRequest = vi.fn();
    const result = await handlePromptRequest(
      {
        title: "Delegate through gateway",
        description: "Ensure the TS entrypoint talks to the Rust engine",
      },
      {
        loadTemplate: async () => [
          "Feature: {{title}}",
          "{{description_or_default}}",
          "Labels: {{labels_csv}}",
        ].join("\n"),
        provider,
        logRequest,
      },
    );

    expect(fetchSpy).toHaveBeenCalledTimes(1);
    const call = (fetchSpy as unknown as { mock: { calls: unknown[][] } }).mock.calls[0];
    expect(call[0]).toBe("http://127.0.0.1:8042/v1/completions");
    expect(result.provider).toBe("llama.cpp");
    expect(result.completion).toBe("Rust engine: completion");
    expect(logRequest).toHaveBeenCalledWith(
      expect.objectContaining({ provider: "llama.cpp", status: "success" }),
    );
  });
});
