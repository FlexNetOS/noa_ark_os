# Host Control Token Surface

The host control surface combines the capability token service with isolation-aware
APIs that govern environment takeover and resource arbitration. Each host level
operation requires explicit scopes defined in the kernel manifest, ensuring
security decisions are rooted in configuration.

## Token Scopes

The default manifest introduces two host-control scopes:

- `host.environment.takeover` – Grants the ability to obtain a lease over a
  managed environment. Only one active lease may exist per environment and the
  token holder must release it when complete.
- `host.resource.arbitrate` – Permits arbitration of CPU and memory allocations
  for environments already leased by the requesting token.

Both scopes map to the security and process subsystems, and tokens are capped by
policy TTL values derived from the manifest.

## Service Flow

1. `KernelManifest` loads token policies and the kernel wires them into the
   global token service during startup.
2. The security subsystem issues tokens via `SecurityService::issue_scope_token`
   which delegates to the global token service.
3. `HostControlService` validates the supplied token scopes before granting
   environment leases or resource envelopes, ensuring isolation between actors.

## Harness & Tests

- `cargo run -p noa_core --bin noa_host_control` executes the harness that
  demonstrates environment takeover, arbitration, and release.
- Integration tests under `core/tests/` validate token issuance, revocation, and
  cross-token isolation for host control operations.
