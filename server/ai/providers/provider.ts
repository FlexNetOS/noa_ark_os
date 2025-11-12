/**
 * Provides a small abstraction for AI providers used by the Kanban AI assist feature.
 * Imported by the Next.js API route via server/ai/router.ts.
 */

export type ProviderContext = {
  signal?: AbortSignal;
};

export interface AIProvider {
  readonly name: string;
  isConfigured(): boolean;
  completePrompt(prompt: string, context?: ProviderContext): Promise<string>;
}

export class ProviderConfigurationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ProviderConfigurationError";
  }
}

export class ProviderRequestError extends Error {
  constructor(message: string, readonly status?: number) {
    super(message);
    this.name = "ProviderRequestError";
  }
}
