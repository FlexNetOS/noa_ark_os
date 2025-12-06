import test from "node:test";
import assert from "node:assert";
import { getProvider, resetProviderCache } from "../../../server/ai/router.ts";

test("defaults to llama provider", () => {
  resetProviderCache();
  const provider = getProvider({} as NodeJS.ProcessEnv);
  assert.ok(provider, "provider should be resolved");
  assert.strictEqual(provider?.name, "llama.cpp");
});

test("selects explicit provider", () => {
  resetProviderCache();
  const env = {
    AI_PROVIDER: "openai",
    OPENAI_API_KEY: "test-key",
  } as unknown as NodeJS.ProcessEnv;
  const provider = getProvider(env);
  assert.ok(provider);
  assert.strictEqual(provider?.name, "openai");
});
