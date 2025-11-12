/**
 * Provides a lightweight SQLite logger for AI assist requests.
 * Used by the /api/ai/prompt route to persist telemetry.
 */

import Database from "better-sqlite3";
import { mkdirSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

export type AiRequestLog = {
  id: number;
  createdAt: string;
  source: string;
  cardId?: string | null;
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

  constructor(private readonly dbPath: string, private readonly migrationsDir: string) {
    mkdirSync(dirname(dbPath), { recursive: true });
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
  }

  private applyMigrations() {
    this.db.exec(
      `CREATE TABLE IF NOT EXISTS schema_migrations (
        id TEXT PRIMARY KEY
      )`
    );

    const appliedRows = this.db
      .prepare(`SELECT id FROM schema_migrations`)
      .all() as Array<{ id: string }>;
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
    cardId?: string;
    title: string;
    provider: string | null;
    latencyMs: number;
    status: "success" | "error";
    errorMsg?: string;
  }) {
    this.insertStatement.run({
      source: entry.source,
      cardId: entry.cardId ?? null,
      title: entry.title,
      provider: entry.provider ?? null,
      latencyMs: entry.latencyMs,
      status: entry.status,
      errorMsg: entry.errorMsg ?? null,
    });
  }

  listRecent(limit = 50): AiRequestLog[] {
    return this.listStatement.all({ limit }) as AiRequestLog[];
  }
}

const DATA_DIRECTORY = resolve(process.cwd(), ".data");
const DB_PATH = join(DATA_DIRECTORY, "ai_assist.sqlite");
const MIGRATIONS_DIRECTORY = resolve(process.cwd(), "server/migrations");

const globalAny = globalThis as typeof globalThis & { __aiDatabase?: AiDatabase };

if (!globalAny.__aiDatabase) {
  globalAny.__aiDatabase = new AiDatabase(DB_PATH, MIGRATIONS_DIRECTORY);
}

export const aiDatabase = globalAny.__aiDatabase;

export function listRecentAiRequests(limit?: number) {
  return aiDatabase.listRecent(limit);
}
