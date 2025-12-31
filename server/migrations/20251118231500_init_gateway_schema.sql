-- Migration: initialize gateway relational schema for capability tokens, audit traces, and rate limit state.
-- All structures follow AGENT.md requirements: auditable, append-only, and ready for offline operation.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS capability_tokens (
    token_id TEXT PRIMARY KEY,
    issued_to TEXT NOT NULL,
    scopes TEXT[] NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS capability_tokens_issued_to_idx ON capability_tokens (issued_to);
CREATE INDEX IF NOT EXISTS capability_tokens_expires_at_idx ON capability_tokens (expires_at);

CREATE TABLE IF NOT EXISTS gateway_traces (
    id BIGSERIAL PRIMARY KEY,
    request_id TEXT NOT NULL,
    user_id BIGINT,
    agent_id TEXT,
    protocol TEXT NOT NULL,
    route_targets TEXT[] NOT NULL DEFAULT '{}'::text[],
    decision JSONB NOT NULL DEFAULT '{}'::jsonb,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS gateway_traces_request_id_idx ON gateway_traces (request_id);
CREATE INDEX IF NOT EXISTS gateway_traces_recorded_at_idx ON gateway_traces (recorded_at);

CREATE TABLE IF NOT EXISTS rate_limit_state (
    agent_id TEXT PRIMARY KEY,
    layer TEXT NOT NULL,
    remaining INTEGER NOT NULL,
    last_refill TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS rate_limit_state_layer_idx ON rate_limit_state (layer);
