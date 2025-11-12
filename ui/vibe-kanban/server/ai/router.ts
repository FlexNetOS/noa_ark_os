import type { AIProvider } from "./types";

class DisabledProvider implements AIProvider {
  name = "disabled";

  isConfigured() {
    return false;
  }

  async completePrompt(_prompt: string): Promise<string> {
    throw new Error("AI provider is not configured");
  }
}

const disabledProvider = new DisabledProvider();

export function getProvider(_env: NodeJS.ProcessEnv = process.env): AIProvider | null {
  return disabledProvider.isConfigured() ? disabledProvider : null;
}
