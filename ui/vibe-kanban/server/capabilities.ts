import fs from "fs/promises";
import path from "path";

import {
  DEFAULT_CAPABILITY_REGISTRY,
  type CapabilityRegistry,
  normalizeCapabilityRegistry,
} from "@/shared/capabilities";

const CAPABILITY_REGISTRY_PATH = path.resolve(
  process.cwd(),
  "..",
  "..",
  "registry",
  "capabilities.json"
);

let cachedRegistry: CapabilityRegistry | null = null;
let loadPromise: Promise<CapabilityRegistry> | null = null;

function cloneDefaultRegistry(): CapabilityRegistry {
  return { version: DEFAULT_CAPABILITY_REGISTRY.version, capabilities: [] };
}

// Type guard for Node.js error with code property
function isNodeError(error: unknown): error is NodeJS.ErrnoException {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error
  );
}

async function loadCapabilityRegistryFromDisk(): Promise<CapabilityRegistry> {
  try {
    const raw = await fs.readFile(CAPABILITY_REGISTRY_PATH, "utf-8");
    const parsed = JSON.parse(raw) as unknown;
    return normalizeCapabilityRegistry(parsed);
  } catch (error) {
    if (!isNodeError(error) || error.code !== "ENOENT") {
      console.warn(
        `Failed to load capability registry from ${CAPABILITY_REGISTRY_PATH}:`,
        error
      );
    }
    return cloneDefaultRegistry();
  }
}

function ensureLoadPromise(): Promise<CapabilityRegistry> {
  if (!loadPromise) {
    loadPromise = loadCapabilityRegistryFromDisk()
      .then((registry) => {
        cachedRegistry = registry;
        return registry;
      })
      .catch((error) => {
        console.error("Unexpected capability registry failure", error);
        cachedRegistry = cloneDefaultRegistry();
        return cachedRegistry;
      });
  }
  return loadPromise;
}

export async function getCapabilityRegistry(): Promise<CapabilityRegistry> {
  if (cachedRegistry) {
    return cachedRegistry;
  }
  return ensureLoadPromise();
}

export function getCapabilityTokens(registry: CapabilityRegistry): Set<string> {
  const tokens = new Set<string>();
  for (const capability of registry.capabilities) {
    tokens.add(capability.id);
    for (const provided of capability.provides ?? []) {
      tokens.add(provided);
    }
  }
  return tokens;
}

export function __resetCapabilityRegistryCacheForTests(): void {
  cachedRegistry = null;
  loadPromise = null;
}

void ensureLoadPromise();
