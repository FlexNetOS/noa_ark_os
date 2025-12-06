export interface AIProvider {
  name: string;
  isConfigured(): boolean;
  completePrompt(prompt: string): Promise<string>;
}
