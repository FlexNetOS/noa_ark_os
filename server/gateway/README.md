# NOA Gateway

> One policy, one gateway, many engines.

This crate implements the programmable gateway that every surface inside NOA ARK OS must traverse before reaching any AI or
agentic capability. The guiding principle is simple: **a single TypeScript gateway exports policy enforcement, routing, and
observability, while specialised engines (Rust, Python, C++) sit behind it as interchangeable workers**.

## Pattern Overview

1. **One policy** – `server/gateway/src/policy.rs` defines the canonical enforcement logic. All API surfaces rely on the same
   policy decisions so feature flags and auth rules stay consistent across applications.
2. **One gateway** – The TypeScript entrypoint (`@noa-ark/server/ai/router`) exposes a stable interface that UI packages
   consume. It does not implement business logic itself; instead, it orchestrates downstream engines and records telemetry.
3. **Many engines** – The Rust inference router (`server/ai/inference`) is treated as one of many execution engines. The TS
   gateway selects an engine, forwards the request, and normalises the response so front-ends never bind to engine details.

## Why it matters

- **No duplication** – UI packages keep thin wrappers (re-export files) that point back to the shared gateway. When engine
  implementations evolve, only the gateway changes.
- **Deterministic testing** – Both the TS gateway and the Rust engines ship with local unit tests so CI can validate the full
  delegation path without external infrastructure.
- **Drop-in extensibility** – Adding a new engine (for example, a policy-aware summariser) only requires wiring it into the
  gateway router. UI packages continue to import the same entrypoint.

Always route new capabilities through this gateway first, then reach for additional engines behind it.
