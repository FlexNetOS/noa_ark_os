import { readFile } from "node:fs/promises";
import path from "node:path";

export type GoalAgentMetric = {
  agent: string;
  totalRuns: number;
  successfulRuns: number;
  successRate: number;
};

export type GoalMetricSnapshot = {
  goalId: string;
  workflowId: string;
  totalRuns: number;
  successfulRuns: number;
  averageLeadTimeMs: number;
  successRate: number;
  agents: GoalAgentMetric[];
  updatedAt: string;
  contextPenaltyScore: number;
  contextP95Bytes: number;
  contextP95LatencyMs: number;
  rewardTotal: number;
  rewardAverage: number;
  rewardRecent: number;
};

function candidatePaths(): string[] {
  const explicitRoot = process.env.NOA_WORKFLOW_ROOT;
  const fromApp = path.resolve(process.cwd(), "..", "..");
  const fromWorkspace = path.resolve(process.cwd(), "..");
  const targets = [
    explicitRoot ? path.join(explicitRoot, "storage/db/analytics/goal_kpis.json") : null,
    path.join(fromApp, "storage/db/analytics/goal_kpis.json"),
    path.join(fromWorkspace, "storage/db/analytics/goal_kpis.json"),
  ];
  return targets.filter((value): value is string => Boolean(value));
}

export async function readGoalAnalytics(): Promise<GoalMetricSnapshot[]> {
  for (const candidate of candidatePaths()) {
    try {
      const raw = await readFile(candidate, "utf8");
      if (!raw.trim()) {
        continue;
      }
      const parsed = JSON.parse(raw) as unknown;
      if (!Array.isArray(parsed)) {
        continue;
      }
      return parsed
        .map((entry) => normaliseGoalMetric(entry))
        .filter((entry): entry is GoalMetricSnapshot => Boolean(entry));
    } catch (error) {
      const err = error as NodeJS.ErrnoException;
      if (err.code === "ENOENT") {
        continue;
      }
      console.warn("Failed to read goal analytics", candidate, err);
    }
  }
  return [];
}

export async function findGoalMetric(goalId: string): Promise<GoalMetricSnapshot | undefined> {
  const metrics = await readGoalAnalytics();
  return metrics.find((entry) => entry.goalId === goalId);
}

function normaliseGoalMetric(input: unknown): GoalMetricSnapshot | undefined {
  if (!input || typeof input !== "object") {
    return undefined;
  }
  const raw = input as Partial<GoalMetricSnapshot> & {
    agents?: unknown;
  };
  const agents = Array.isArray(raw.agents)
    ? raw.agents
        .map((agent) => normaliseAgentMetric(agent))
        .filter((agent): agent is GoalAgentMetric => Boolean(agent))
    : [];
  if (!raw.goalId || typeof raw.goalId !== "string") {
    return undefined;
  }
  const totalRuns = typeof raw.totalRuns === "number" ? raw.totalRuns : 0;
  const successfulRuns = typeof raw.successfulRuns === "number" ? raw.successfulRuns : 0;
  const averageLeadTimeMs = typeof raw.averageLeadTimeMs === "number" ? raw.averageLeadTimeMs : 0;
  const successRate = typeof raw.successRate === "number" ? raw.successRate : totalRuns > 0 ? successfulRuns / totalRuns : 0;
  const contextPenaltyScore = typeof raw.contextPenaltyScore === "number" ? raw.contextPenaltyScore : 0;
  const contextP95Bytes = typeof raw.contextP95Bytes === "number" ? Math.trunc(raw.contextP95Bytes) : 0;
  const contextP95LatencyMs =
    typeof raw.contextP95LatencyMs === "number" ? Math.trunc(raw.contextP95LatencyMs) : 0;
  const rewardTotal = typeof raw.rewardTotal === "number" ? raw.rewardTotal : 0;
  const rewardAverage = typeof raw.rewardAverage === "number" ? raw.rewardAverage : 0;
  const rewardRecent = typeof raw.rewardRecent === "number" ? raw.rewardRecent : 0;
  return {
    goalId: raw.goalId,
    workflowId: typeof raw.workflowId === "string" ? raw.workflowId : raw.goalId,
    totalRuns,
    successfulRuns,
    averageLeadTimeMs,
    successRate,
    agents,
    updatedAt: typeof raw.updatedAt === "string" ? raw.updatedAt : new Date().toISOString(),
    contextPenaltyScore,
    contextP95Bytes,
    contextP95LatencyMs,
    rewardTotal,
    rewardAverage,
    rewardRecent,
  };
}

function normaliseAgentMetric(input: unknown): GoalAgentMetric | undefined {
  if (!input || typeof input !== "object") {
    return undefined;
  }
  const raw = input as Partial<GoalAgentMetric>;
  if (!raw.agent || typeof raw.agent !== "string") {
    return undefined;
  }
  const totalRuns = typeof raw.totalRuns === "number" ? raw.totalRuns : 0;
  const successfulRuns = typeof raw.successfulRuns === "number" ? raw.successfulRuns : 0;
  const successRate = typeof raw.successRate === "number"
    ? raw.successRate
    : totalRuns > 0
      ? successfulRuns / totalRuns
      : 0;
  return {
    agent: raw.agent,
    totalRuns,
    successfulRuns,
    successRate,
  };
}
