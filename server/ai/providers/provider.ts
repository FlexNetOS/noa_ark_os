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

const DEFAULT_CACHE_CAPACITY = 32;

export class PromptCache {
  private readonly entries = new Map<string, string>();

  constructor(private readonly capacity = DEFAULT_CACHE_CAPACITY) {}

  get(key: string): string | undefined {
    const value = this.entries.get(key);
    if (value === undefined) {
      return undefined;
    }
    this.entries.delete(key);
    this.entries.set(key, value);
    return value;
  }

  set(key: string, value: string): void {
    if (this.entries.has(key)) {
      this.entries.delete(key);
    }
    this.entries.set(key, value);
    if (this.entries.size <= this.capacity) {
      return;
    }
    const oldestKey = this.entries.keys().next().value as string | undefined;
    if (oldestKey) {
      this.entries.delete(oldestKey);
    }
  }
}

export class CachedProvider implements AIProvider {
  readonly name: string;

  constructor(private readonly inner: AIProvider, private readonly cache = new PromptCache()) {
    this.name = inner.name;
  }

  isConfigured(): boolean {
    return this.inner.isConfigured();
  }

  async completePrompt(prompt: string, context?: ProviderContext): Promise<string> {
    const cacheKey = JSON.stringify({ provider: this.name, prompt });
    const cached = this.cache.get(cacheKey);
    if (cached !== undefined) {
      return cached;
    }

    const response = await this.inner.completePrompt(prompt, context);
    this.cache.set(cacheKey, response);
    return response;
  }
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
