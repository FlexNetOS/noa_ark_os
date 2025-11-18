export type CapabilityRecord = {
  id: string;
  type: string;
  semver: string;
  requires?: string[];
  provides?: string[];
};

export type CapabilityRegistry = {
  version: number;
  capabilities: CapabilityRecord[];
};

export const DEFAULT_CAPABILITY_REGISTRY: CapabilityRegistry = Object.freeze({
  version: 1,
  capabilities: [],
});

export type CapabilityFeatureGate = {
  id: string;
  capability: string;
  label: string;
  description: string;
};

export type CapabilityFeatureGateStatus = CapabilityFeatureGate & {
  available: boolean;
};

export const KANBAN_FEATURE_GATES: CapabilityFeatureGate[] = [
  {
    id: "kanban.manageColumns",
    capability: "kanban.manageColumns",
    label: "Column management",
    description: "Create, rename, and remove workflow columns.",
  },
  {
    id: "kanban.quickComposer",
    capability: "kanban.quickComposer",
    label: "Quick composer",
    description: "Draft new cards inline without opening the editor.",
  },
  {
    id: "kanban.metrics",
    capability: "kanban.metrics",
    label: "Advanced metrics",
    description: "Display velocity, flow efficiency, and cycle-time insights.",
  },
  {
    id: "kanban.goalInsights",
    capability: "kanban.goalInsights",
    label: "Goal insights",
    description: "Surface goal-level success rates and lead-time analytics.",
  },
  {
    id: "kanban.assist",
    capability: "kanban.assist",
    label: "Spark assist agent",
    description: "Request AI suggestions tailored to the active board.",
  },
  {
    id: "kanban.autonomousRetry",
    capability: "kanban.autonomousRetry",
    label: "Autonomous retry",
    description: "Automatically re-plan workflows when performance degrades.",
  },
  {
    id: "kanban.agentEscalation",
    capability: "kanban.agentEscalation",
    label: "Agent escalation",
    description: "Escalate complex goals to senior agents when thresholds are breached.",
  },
];

export function normalizeCapabilityRegistry(input: unknown): CapabilityRegistry {
  if (!input || typeof input !== "object") {
    return { ...DEFAULT_CAPABILITY_REGISTRY };
  }

  const raw = input as Partial<CapabilityRegistry> & { capabilities?: unknown };
  const registry: CapabilityRegistry = {
    version: typeof raw.version === "number" ? raw.version : DEFAULT_CAPABILITY_REGISTRY.version,
    capabilities: Array.isArray(raw.capabilities)
      ? raw.capabilities
          .map((item) => normalizeCapabilityRecord(item))
          .filter((item): item is CapabilityRecord => Boolean(item))
      : [],
  };

  return registry;
}

function normalizeCapabilityRecord(input: unknown): CapabilityRecord | null {
  if (!input || typeof input !== "object") {
    return null;
  }

  const raw = input as Partial<CapabilityRecord> & { requires?: unknown; provides?: unknown };
  if (!raw.id || typeof raw.id !== "string") {
    return null;
  }

  return {
    id: raw.id,
    type: typeof raw.type === "string" ? raw.type : "unknown",
    semver: typeof raw.semver === "string" ? raw.semver : "0.0.0",
    requires: Array.isArray(raw.requires) ? raw.requires.map((value) => String(value)) : undefined,
    provides: Array.isArray(raw.provides) ? raw.provides.map((value) => String(value)) : undefined,
  };
}

export function evaluateFeatureGates(
  registry: CapabilityRegistry,
  gates: CapabilityFeatureGate[] = KANBAN_FEATURE_GATES,
): CapabilityFeatureGateStatus[] {
  const provided = new Set<string>();

  for (const capability of registry.capabilities) {
    provided.add(capability.id);
    for (const token of capability.provides ?? []) {
      provided.add(token);
    }
  }

  return gates.map((gate) => ({
    ...gate,
    available: provided.has(gate.capability),
  }));
}
