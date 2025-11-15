export const FEATURE_FLAGS = {
  ambientBackdrop: true,
  quickComposer: true,
  boardMetrics: true,
  goalInsights: true,
  autonomousRetry: true,
  agentEscalation: true,
} as const;

export type FeatureFlagName = keyof typeof FEATURE_FLAGS;

export function isFeatureEnabled(flag: FeatureFlagName) {
  return FEATURE_FLAGS[flag];
}
