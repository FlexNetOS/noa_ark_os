/**
 * Thin wrapper that re-exports the canonical AI gateway router.
 * Keeping this file preserves historical imports while ensuring
 * all traffic flows through the shared @noa-ark/server entrypoint.
 */
export { getProvider, resetProviderCache } from "@noa-ark/server/ai/router";
export type { AIProvider } from "@noa-ark/server/ai/providers/provider";
