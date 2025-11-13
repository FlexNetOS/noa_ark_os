/**
 * Implements a llama.cpp provider using its OpenAI compatible HTTP surface.
 * Wired through server/ai/router.ts and ultimately invoked by the Kanban API route.
 */

import { AIProvider, ProviderConfigurationError, ProviderRequestError, ProviderContext } from "./provider";

export class LlamaCppProvider implements AIProvider {
  readonly name = "llama.cpp";
  private readonly endpoint: string | undefined;
  private readonly fetchImpl: typeof fetch;

  constructor(
    private readonly env: NodeJS.ProcessEnv = process.env,
    options: { fetchImpl?: typeof fetch } = {}
  ) {
    // LLAMA_CPP_ENDPOINT should be the base URL including the '/v1' suffix, e.g. 'http://127.0.0.1:8080/v1'
    // Do NOT include '/completions' in the endpoint; it will be appended automatically.
    this.endpoint = this.env.LLAMA_CPP_ENDPOINT ?? "http://127.0.0.1:8080/v1";
    // Validate endpoint format at construction time
    if (!this.endpoint.endsWith("/v1")) {
      throw new ProviderConfigurationError(
        `LLAMA_CPP_ENDPOINT must end with '/v1' (e.g. 'http://127.0.0.1:8080/v1'). Got: '${this.endpoint}'`
      );
    }
    this.fetchImpl = options.fetchImpl ?? fetch;
  }

  isConfigured(): boolean {
    return typeof this.endpoint === "string" && this.endpoint.length > 0;
  }

  async completePrompt(prompt: string, context?: ProviderContext): Promise<string> {
    if (!this.isConfigured()) {
      throw new ProviderConfigurationError("LLAMA_CPP_ENDPOINT is not configured");
    }

    const response = await this.fetchImpl(`${this.endpoint!.replace(/\/$/, "")}/completions`, {
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
