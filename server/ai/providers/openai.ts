/**
 * Implements an OpenAI provider using the Chat Completions API.
 * Consumed by the Kanban AI assist HTTP controller.
 */

import { AIProvider, ProviderConfigurationError, ProviderRequestError, ProviderContext } from "./provider";

export class OpenAIProvider implements AIProvider {
  readonly name = "openai";
  private readonly apiKey: string | undefined;
  private readonly baseUrl: string;
  private readonly model: string;

  constructor(private readonly env: NodeJS.ProcessEnv = process.env) {
    this.apiKey = this.env.OPENAI_API_KEY;
    this.baseUrl = this.env.OPENAI_BASE_URL ?? "https://api.openai.com/v1";
    this.model = this.env.OPENAI_MODEL ?? "gpt-4o-mini";
  }

  isConfigured(): boolean {
    return typeof this.apiKey === "string" && this.apiKey.length > 0;
  }

  async completePrompt(prompt: string, context?: ProviderContext): Promise<string> {
    if (!this.isConfigured()) {
      throw new ProviderConfigurationError("OPENAI_API_KEY is not configured");
    }

    const response = await fetch(`${this.baseUrl.replace(/\/$/, "")}/chat/completions`, {
      method: "POST",
      headers: {
        authorization: `Bearer ${this.apiKey}`,
        "content-type": "application/json",
      },
      body: JSON.stringify({
        model: this.model,
        messages: [
          { role: "system", content: "You are a precise software engineering assistant." },
          { role: "user", content: prompt },
        ],
        temperature: 0.2,
      }),
      signal: context?.signal,
    });

    if (!response.ok) {
      throw new ProviderRequestError(`OpenAI request failed with status ${response.status}`, response.status);
    }

    const data = (await response.json()) as { choices?: Array<{ message?: { content?: string } }> };
    const text = data.choices?.[0]?.message?.content;
    if (!text) {
      throw new ProviderRequestError("OpenAI response did not contain a message");
    }
    return text.trim();
  }
}
