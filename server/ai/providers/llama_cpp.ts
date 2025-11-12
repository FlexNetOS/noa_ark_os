/**
 * Implements a llama.cpp provider using its OpenAI compatible HTTP surface.
 * Wired through server/ai/router.ts and ultimately invoked by the Kanban API route.
 */

import { AIProvider, ProviderConfigurationError, ProviderRequestError, ProviderContext } from "./provider";

export class LlamaCppProvider implements AIProvider {
  readonly name = "llama.cpp";
  private readonly endpoint: string | undefined;

  constructor(private readonly env: NodeJS.ProcessEnv = process.env) {
    this.endpoint = this.env.LLAMA_CPP_ENDPOINT;
  }

  isConfigured(): boolean {
    return typeof this.endpoint === "string" && this.endpoint.length > 0;
  }

  async completePrompt(prompt: string, context?: ProviderContext): Promise<string> {
    if (!this.isConfigured()) {
      throw new ProviderConfigurationError("LLAMA_CPP_ENDPOINT is not configured");
    }

    const response = await fetch(`${this.endpoint!.replace(/\/$/, "")}/completions`, {
      method: "POST",
      headers: {
        "content-type": "application/json",
      },
      body: JSON.stringify({
        prompt,
        max_tokens: 512,
        temperature: 0.2,
        stream: false,
      }),
      signal: context?.signal,
    });

    if (!response.ok) {
      throw new ProviderRequestError(`llama.cpp request failed with status ${response.status}`, response.status);
    }

    const data = (await response.json()) as { choices?: Array<{ text?: string }> };
    const text = data.choices?.[0]?.text;
    if (!text) {
      throw new ProviderRequestError("llama.cpp response did not contain text");
    }
    return text.trim();
  }
}
