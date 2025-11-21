-- Migration: canonical gateway schema (0001_init)
-- Captures capability tokens, gateway traces, rate-limit state, and telemetry buffers.
-- This file is the canonical starting point for sqlx migrations under server/migrations.

-- Cryptographic helpers for token generation and secure identifiers.
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Capability tokens issued by the kernel/gateway.
CREATE TABLE IF NOT EXISTS capability_tokens (
    token_id   TEXT PRIMARY KEY,
    issued_to  TEXT NOT NULL,
    scopes     TEXT[] NOT NULL,
    issued_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    metadata   JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS capability_tokens_issued_to_idx
    ON capability_tokens (issued_to);

CREATE INDEX IF NOT EXISTS capability_tokens_expires_at_idx
    ON capability_tokens (expires_at);

-- Gateway request/response traces for auditability.
CREATE TABLE IF NOT EXISTS gateway_traces (
    id            BIGSERIAL PRIMARY KEY,
    request_id    TEXT NOT NULL,
    user_id       BIGINT,
    agent_id      TEXT,
    protocol      TEXT NOT NULL,
    route_targets TEXT[] NOT NULL DEFAULT '{}'::text[],
    decision      JSONB NOT NULL DEFAULT '{}'::jsonb,
    recorded_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS gateway_traces_request_id_idx
    ON gateway_traces (request_id);

CREATE INDEX IF NOT EXISTS gateway_traces_recorded_at_idx
    ON gateway_traces (recorded_at);

-- Layer-aware rate limit state, mirroring the gateway's persistent quota store.
CREATE TABLE IF NOT EXISTS rate_limit_state (
    agent_id    TEXT PRIMARY KEY,
    layer       TEXT NOT NULL,
    remaining   INTEGER NOT NULL,
    last_refill TIMESTAMPTZ NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS rate_limit_state_layer_idx
    ON rate_limit_state (layer);

-- Telemetry buffers: append-only spool for gateway and node telemetry payloads.
-- Each row represents a discrete, immutable telemetry chunk (event, metrics snapshot, etc.).
CREATE TABLE IF NOT EXISTS telemetry_buffers (
    id          BIGSERIAL PRIMARY KEY,
    source      TEXT NOT NULL,          -- logical source (node id, profile name, gateway instance)
    kind        TEXT NOT NULL,          -- e.g. 'gateway_event', 'gateway_metrics', 'service_health'
    sequence    BIGINT NOT NULL,        -- monotonic sequence number from the emitter, if available
    payload     JSONB NOT NULL,         -- opaque telemetry payload (mirrors TelemetryEvent / GatewayMetrics)
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS telemetry_buffers_source_kind_sequence_idx
    ON telemetry_buffers (source, kind, sequence);

CREATE INDEX IF NOT EXISTS telemetry_buffers_received_at_idx
    ON telemetry_buffers (received_at);

