/**
 * Implements an Anthropic provider using the Messages API for Claude models.
 * Loaded dynamically based on AI_PROVIDER configuration.
 */

import { AIProvider, ProviderConfigurationError, ProviderRequestError, ProviderContext } from "./provider";

export class AnthropicProvider implements AIProvider {
  readonly name = "anthropic";
  private readonly apiKey: string | undefined;
  private readonly baseUrl: string;
  private readonly model: string;
  private readonly fetchImpl: typeof fetch;

  constructor(
    private readonly env: NodeJS.ProcessEnv = process.env,
    options: { fetchImpl?: typeof fetch } = {}
  ) {
    this.apiKey = this.env.ANTHROPIC_API_KEY;
    this.baseUrl = this.env.ANTHROPIC_BASE_URL ?? "https://api.anthropic.com/v1";
    this.model = this.env.ANTHROPIC_MODEL ?? "claude-3-haiku-20240307";
    this.fetchImpl = options.fetchImpl ?? fetch;
  }

  isConfigured(): boolean {
    return typeof this.apiKey === "string" && this.apiKey.length > 0;
  }

  async completePrompt(prompt: string, context?: ProviderContext): Promise<string> {
    if (!this.isConfigured()) {
      throw new ProviderConfigurationError("ANTHROPIC_API_KEY is not configured");
    }

    const response = await this.fetchImpl(`${this.baseUrl.replace(/\/$/, "")}/messages`, {
      method: "POST",
      headers: {
        "x-api-key": this.apiKey!,
        "anthropic-version": "2023-06-01",
        "content-type": "application/json",
      },
      body: JSON.stringify({
        model: this.model,
        max_tokens: 1024,
        temperature: 0.2,
        messages: [
          { role: "user", content: prompt },
        ],
      }),
      signal: context?.signal,
    });

    if (!response.ok) {
      throw new ProviderRequestError(`Anthropic request failed with status ${response.status}`, response.status);
    }

    const data = (await response.json()) as { content?: Array<{ text?: string }> };
    const text = data.content?.[0]?.text;
    if (!text) {
      throw new ProviderRequestError("Anthropic response did not contain text");
    }
    return text.trim();
  }
}
