-- Creates the ai_requests table for logging Kanban AI interactions.
CREATE TABLE IF NOT EXISTS ai_requests (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  source TEXT NOT NULL,
  card_id TEXT,
  title TEXT NOT NULL,
  provider TEXT,
  latency_ms INTEGER NOT NULL,
  status TEXT NOT NULL,
  error_msg TEXT
);

CREATE INDEX IF NOT EXISTS idx_ai_requests_created_at ON ai_requests(created_at DESC);
