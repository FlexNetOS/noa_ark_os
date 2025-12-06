const DEFAULTS = {
  OFFLINE_FIRST: "true",
  ONLINE_GITHUB_MODE: "false",
  AI_PROVIDER: "llama.cpp",
};

export type EnvironmentState = {
  offlineFirst: boolean;
  onlineMode: boolean;
  provider: string;
};

export function readEnv(env: NodeJS.ProcessEnv = process.env): EnvironmentState {
  return {
    offlineFirst: (env.OFFLINE_FIRST ?? DEFAULTS.OFFLINE_FIRST).toLowerCase() !== "false",
    onlineMode: (env.ONLINE_GITHUB_MODE ?? DEFAULTS.ONLINE_GITHUB_MODE).toLowerCase() === "true",
    provider: (env.AI_PROVIDER ?? DEFAULTS.AI_PROVIDER).toLowerCase(),
  };
}
