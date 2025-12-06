/**
 * Selects the active AI provider based on environment configuration.
 * The Next.js API route imports this router to satisfy provider selection requirements.
 */

import { AIProvider, CachedProvider } from "./providers/provider";
import { LlamaCppProvider } from "./providers/llama_cpp";
import { OpenAIProvider } from "./providers/openai";
import { AnthropicProvider } from "./providers/anthropic";

const providerCache = new Map<string, AIProvider>();

function instantiateProvider(kind: string, env: NodeJS.ProcessEnv) {
  switch (kind) {
    case "llama.cpp":
      return new LlamaCppProvider(env);
    case "openai":
      return new OpenAIProvider(env);
    case "anthropic":
      return new AnthropicProvider(env);
    default:
      return undefined;
  }
}

export function getProvider(env: NodeJS.ProcessEnv = process.env): AIProvider | null {
  const requested = (env.AI_PROVIDER ?? "llama.cpp").toLowerCase();

  if (providerCache.has(requested)) {
    const cached = providerCache.get(requested)!;
    return cached.isConfigured() ? cached : null;
  }

  const instance = instantiateProvider(requested, env);
  if (!instance) {
    return null;
  }

  if (!instance.isConfigured()) {
    providerCache.set(requested, instance);
    return null;
  }

  const cachedProvider = new CachedProvider(instance);
  providerCache.set(requested, cachedProvider);
  return cachedProvider;
}

export function resetProviderCache(): void {
  providerCache.clear();
}
