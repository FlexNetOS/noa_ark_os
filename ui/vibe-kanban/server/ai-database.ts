/**
 * Provides a lightweight SQLite logger for AI assist requests.
 * Used by the /api/ai/prompt route to persist telemetry.
 */

import Database from "better-sqlite3";
import { existsSync, mkdirSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { logInfo, logWarn } from "@noa-ark/shared-ui/logging";

export type GoalLifecycleEvent = {
  id: number;
  goalId: string;
  workspaceId: string;
  eventType: string;
  status?: string | null;
  summary?: string | null;
  payload?: unknown;
  createdAt: string;
};

export type AgentArtifactRecord = {
  id: number;
  goalId: string;
  workspaceId: string;
  artifactType: string;
  artifactUri: string;
  title?: string | null;
  summary?: string | null;
  metadata?: Record<string, unknown> | null;
  createdAt: string;
};

export type GoalEmbeddingRecord = {
  id: number;
  goalId: string;
  workspaceId: string;
  embeddingModel: string;
  embedding: number[];
  embeddingNorm: number;
  createdAt: string;
  updatedAt: string;
};

export type AiRequestLog = {
  id: number;
  createdAt: string;
  source: string;
  cardId?: string | null;
  goalId?: string | null;
  title: string;
  provider?: string | null;
  latencyMs: number;
  status: string;
  errorMsg?: string | null;
};

class AiDatabase {
  private readonly db: Database.Database;
  private readonly insertStatement: Database.Statement;
  private readonly listStatement: Database.Statement;
  private readonly insertGoalEventStatement: Database.Statement;
  private readonly listGoalEventsStatement: Database.Statement;
  private readonly insertArtifactStatement: Database.Statement;
  private readonly listArtifactsStatement: Database.Statement;
  private readonly upsertEmbeddingStatement: Database.Statement;
  private readonly getEmbeddingStatement: Database.Statement;
  private readonly listEmbeddingsStatement: Database.Statement;

  constructor(private readonly dbPath: string, private readonly migrationsDir: string) {
    mkdirSync(dirname(dbPath), { recursive: true });
    mkdirSync(ARTIFACT_DIRECTORY, { recursive: true });
    this.db = new Database(dbPath);
    this.db.pragma("journal_mode = WAL");
    this.applyMigrations();
    this.insertStatement = this.db.prepare(
      `INSERT INTO ai_requests (source, card_id, title, provider, latency_ms, status, error_msg)
       VALUES (@source, @cardId, @title, @provider, @latencyMs, @status, @errorMsg)`
    );
    this.listStatement = this.db.prepare(
      `SELECT id, created_at as createdAt, source, card_id as cardId, title, provider, latency_ms as latencyMs, status, error_msg as errorMsg
       FROM ai_requests
       ORDER BY created_at DESC
       LIMIT @limit`
    );
    this.insertGoalEventStatement = this.db.prepare(
      `INSERT INTO goal_lifecycle_events (goal_id, workspace_id, event_type, status, summary, payload)
       VALUES (@goalId, @workspaceId, @eventType, @status, @summary, json(@payload))`
    );
    this.listGoalEventsStatement = this.db.prepare(
      `SELECT id, goal_id as goalId, workspace_id as workspaceId, event_type as eventType, status, summary, json_extract(payload, '$') as payload, created_at as createdAt
       FROM goal_lifecycle_events
       WHERE goal_id = @goalId
       ORDER BY created_at DESC
       LIMIT @limit`
    );
    this.insertArtifactStatement = this.db.prepare(
      `INSERT INTO agent_artifacts (goal_id, workspace_id, artifact_type, artifact_uri, title, summary, metadata)
       VALUES (@goalId, @workspaceId, @artifactType, @artifactUri, @title, @summary, json(@metadata))`
    );
    this.listArtifactsStatement = this.db.prepare(
      `SELECT id, goal_id as goalId, workspace_id as workspaceId, artifact_type as artifactType, artifact_uri as artifactUri, title, summary,
              json_extract(metadata, '$') as metadata, created_at as createdAt
       FROM agent_artifacts
       WHERE goal_id = @goalId
       ORDER BY created_at DESC
       LIMIT @limit`
    );
    this.upsertEmbeddingStatement = this.db.prepare(
      `INSERT INTO goal_embeddings (goal_id, workspace_id, embedding_model, embedding, embedding_norm)
       VALUES (@goalId, @workspaceId, @embeddingModel, json(@embedding), @embeddingNorm)
       ON CONFLICT(goal_id, embedding_model)
       DO UPDATE SET embedding = excluded.embedding, embedding_norm = excluded.embedding_norm, updated_at = CURRENT_TIMESTAMP`
    );
    this.getEmbeddingStatement = this.db.prepare(
      `SELECT id, goal_id as goalId, workspace_id as workspaceId, embedding_model as embeddingModel,
              json_extract(embedding, '$') as embedding, embedding_norm as embeddingNorm,
              created_at as createdAt, updated_at as updatedAt
       FROM goal_embeddings
       WHERE goal_id = @goalId AND embedding_model = @embeddingModel`
    );
    this.listEmbeddingsStatement = this.db.prepare(
      `SELECT id, goal_id as goalId, workspace_id as workspaceId, embedding_model as embeddingModel,
              json_extract(embedding, '$') as embedding, embedding_norm as embeddingNorm,
              created_at as createdAt, updated_at as updatedAt
       FROM goal_embeddings`
    );
  }

  private applyMigrations() {
    this.db.exec(
      `CREATE TABLE IF NOT EXISTS schema_migrations (
        id TEXT PRIMARY KEY
      )`
    );

    const appliedRowsLegacy = this.db.prepare(`SELECT id FROM schema_migrations`).all() as Array<{ id: string }>;
    const appliedRows = Array.isArray(appliedRowsLegacy)
      ? appliedRowsLegacy
      : (this.db.prepare(`SELECT id FROM schema_migrations`).all() as Array<{ id: string }>);
    const applied = new Set<string>(appliedRows.map((row) => row.id));

    const migrationFiles = readdirSync(this.migrationsDir)
      .filter((file) => file.endsWith(".sql"))
      .sort();

    for (const file of migrationFiles) {
      if (applied.has(file)) {
        continue;
      }
      const sql = readFileSync(join(this.migrationsDir, file), "utf8");
      this.db.exec("BEGIN");
      try {
        this.db.exec(sql);
        this.db.prepare(`INSERT INTO schema_migrations (id) VALUES (?)`).run(file);
        this.db.exec("COMMIT");
      } catch (error) {
        this.db.exec("ROLLBACK");
        throw error;
      }
    }
  }

  logRequest(entry: {
    source: string;
    goalId?: string;
    cardId?: string;
    title: string;
    provider: string | null;
    latencyMs: number;
    status: "success" | "error";
    errorMsg?: string;
  }) {
    this.insertStatement.run({
      source: entry.source,
      cardId: entry.goalId ?? entry.cardId ?? null,
      title: entry.title,
      provider: entry.provider ?? null,
      latencyMs: entry.latencyMs,
      status: entry.status,
      errorMsg: entry.errorMsg ?? null,
    });
  }

  listRecent(limit = 50): AiRequestLog[] {
    const rows = this.listStatement.all({ limit }) as AiRequestLog[];
    return rows.map((row) => ({ ...row, goalId: row.goalId ?? row.cardId ?? null }));
  }

  recordGoalLifecycleEvent(event: {
    goalId: string;
    workspaceId: string;
    eventType: string;
    status?: string | null;
    summary?: string | null;
    payload?: unknown;
  }) {
    this.insertGoalEventStatement.run({
      goalId: event.goalId,
      workspaceId: event.workspaceId,
      eventType: event.eventType,
      status: event.status ?? null,
      summary: event.summary ?? null,
      payload: event.payload ?? null,
    });
    logInfo({
      component: "ai.database",
      event: "goal_event_recorded",
      message: `Recorded lifecycle event for goal ${event.goalId}`,
      outcome: "success",
      context: {
        workspaceId: event.workspaceId,
        eventType: event.eventType,
        status: event.status ?? undefined,
      },
    });
  }

  listGoalLifecycleEvents(goalId: string, limit = 50): GoalLifecycleEvent[] {
    return this.listGoalEventsStatement.all({ goalId, limit }) as GoalLifecycleEvent[];
  }

  recordArtifact(artifact: {
    goalId: string;
    workspaceId: string;
    artifactType: string;
    artifactUri: string;
    title?: string | null;
    summary?: string | null;
    metadata?: Record<string, unknown> | null;
  }) {
    const artifactPath = ensureArtifactPath(artifact.artifactUri);
    this.insertArtifactStatement.run({
      goalId: artifact.goalId,
      workspaceId: artifact.workspaceId,
      artifactType: artifact.artifactType,
      artifactUri: artifactPath,
      title: artifact.title ?? null,
      summary: artifact.summary ?? null,
      metadata: artifact.metadata ?? null,
    });
    logInfo({
      component: "ai.database",
      event: "goal_artifact_stored",
      message: `Stored artifact for goal ${artifact.goalId}`,
      outcome: "success",
      context: {
        workspaceId: artifact.workspaceId,
        artifactType: artifact.artifactType,
        artifactUri: artifactPath,
      },
    });
  }

  listArtifacts(goalId: string, limit = 50): AgentArtifactRecord[] {
    return this.listArtifactsStatement.all({ goalId, limit }) as AgentArtifactRecord[];
  }

  upsertGoalEmbedding(record: {
    goalId: string;
    workspaceId: string;
    embeddingModel: string;
    embedding: number[];
  }) {
    const norm = Math.sqrt(record.embedding.reduce((sum, value) => sum + value * value, 0));
    this.upsertEmbeddingStatement.run({
      goalId: record.goalId,
      workspaceId: record.workspaceId,
      embeddingModel: record.embeddingModel,
      embedding: record.embedding,
      embeddingNorm: norm,
    });
    logInfo({
      component: "ai.database",
      event: "goal_embedding_upserted",
      message: `Updated embedding for goal ${record.goalId}`,
      outcome: "success",
      context: {
        workspaceId: record.workspaceId,
        embeddingModel: record.embeddingModel,
        vectorLength: record.embedding.length,
      },
    });
  }

  getGoalEmbedding(goalId: string, embeddingModel: string): GoalEmbeddingRecord | undefined {
    const row = this.getEmbeddingStatement.get({ goalId, embeddingModel }) as GoalEmbeddingRecord | undefined;
    if (!row) {
      return undefined;
    }
    return normalizeEmbeddingRow(row);
  }

  searchSimilarGoals(goalId: string, embeddingModel: string, topN = 5) {
    const target = this.getGoalEmbedding(goalId, embeddingModel);
    if (!target) {
      return [] as Array<{ goalId: string; workspaceId: string; score: number }>;
    }
    const all = this.listEmbeddingsStatement.all() as GoalEmbeddingRecord[];
    const matches: Array<{ goalId: string; workspaceId: string; score: number }> = [];
    for (const candidateRow of all) {
      if (candidateRow.goalId === goalId || candidateRow.embeddingModel !== embeddingModel) {
        continue;
      }
      const candidate = normalizeEmbeddingRow(candidateRow);
      const score = cosineSimilarity(target.embedding, target.embeddingNorm, candidate.embedding, candidate.embeddingNorm);
      if (!Number.isFinite(score)) {
        continue;
      }
      matches.push({ goalId: candidate.goalId, workspaceId: candidate.workspaceId, score });
    }
    matches.sort((a, b) => b.score - a.score);
    if (!matches.length) {
      logWarn({
        component: "ai.database",
        event: "goal_embedding_no_matches",
        message: `No similar goals found for ${goalId}`,
        outcome: "informational",
        context: { embeddingModel },
      });
    }
    return matches.slice(0, topN);
  }
}

const DATA_DIRECTORY = resolve(process.cwd(), ".data");
const DB_PATH = join(DATA_DIRECTORY, "ai_assist.sqlite");
const MIGRATIONS_DIRECTORY = resolve(process.cwd(), "server/migrations");
const ARTIFACT_DIRECTORY = resolve(process.cwd(), "storage/artifacts/goals");

function ensureArtifactPath(uri: string) {
  const candidate = uri.startsWith("/") ? uri : join(ARTIFACT_DIRECTORY, uri);
  if (!existsSync(dirname(candidate))) {
    mkdirSync(dirname(candidate), { recursive: true });
  }
  return candidate;
}

function normalizeEmbeddingRow(row: GoalEmbeddingRecord): GoalEmbeddingRecord {
  return {
    ...row,
    embedding: Array.isArray(row.embedding)
      ? (row.embedding as unknown[]).map((value) => Number(value))
      : JSON.parse(String(row.embedding ?? "[]")),
    embeddingNorm: Number(row.embeddingNorm ?? 0),
    createdAt: row.createdAt,
    updatedAt: row.updatedAt,
  };
}

function cosineSimilarity(a: number[], normA: number, b: number[], normB: number) {
  if (!normA || !normB) {
    return 0;
  }
  if (a.length !== b.length) {
    return NaN;
  }
  let dot = 0;
  for (let index = 0; index < a.length; index += 1) {
    dot += a[index] * b[index];
  }
  return dot / (normA * normB);
}

const globalAny = globalThis as typeof globalThis & { __aiDatabase?: AiDatabase };

if (!globalAny.__aiDatabase) {
  globalAny.__aiDatabase = new AiDatabase(DB_PATH, MIGRATIONS_DIRECTORY);
}

export const aiDatabase = globalAny.__aiDatabase;

export function listRecentAiRequests(limit?: number) {
  return aiDatabase.listRecent(limit);
}
