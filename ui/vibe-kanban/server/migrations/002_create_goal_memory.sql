-- Creates tables for goal lifecycle tracking, agent artifacts, and embeddings.
CREATE TABLE IF NOT EXISTS goal_lifecycle_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  goal_id TEXT NOT NULL,
  workspace_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  status TEXT,
  summary TEXT,
  payload TEXT,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_goal_lifecycle_goal_id_created_at
  ON goal_lifecycle_events(goal_id, created_at DESC);

CREATE TABLE IF NOT EXISTS agent_artifacts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  goal_id TEXT NOT NULL,
  workspace_id TEXT NOT NULL,
  artifact_type TEXT NOT NULL,
  artifact_uri TEXT NOT NULL,
  title TEXT,
  summary TEXT,
  metadata TEXT,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_agent_artifacts_goal_id_created_at
  ON agent_artifacts(goal_id, created_at DESC);

CREATE TABLE IF NOT EXISTS goal_embeddings (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  goal_id TEXT NOT NULL,
  workspace_id TEXT NOT NULL,
  embedding_model TEXT NOT NULL,
  embedding TEXT NOT NULL,
  embedding_norm REAL NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(goal_id, embedding_model)
);

CREATE INDEX IF NOT EXISTS idx_goal_embeddings_model
  ON goal_embeddings(embedding_model, goal_id);
