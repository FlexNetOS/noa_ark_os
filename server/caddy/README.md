# Caddy Reverse Proxy Bundle

Caddy fronts the unified server with automatic HTTPS, reverse proxying,
rate limiting, and structured logging exactly as documented in
`server/BUILD_SPEC.md`.

## Contents

- `Caddyfile` – Production template that mirrors the build specification.
- `overlays/` – Environment-specific additions (development TLS, staging
  logging, production mTLS).
- `caddy.env` – Environment helper to export shared variables used by the
  templates.
- Portable tooling (`server/tools/setup-portable-caddy.*`) – Downloads a
  hermetic Caddy binary for Linux, macOS, or Windows.

## Overlay strategy

Each overlay file starts with `import ../Caddyfile` so the base
configuration is always loaded. Additional site blocks then extend the
setup for a specific environment:

- `overlays/dev.caddy` – Adds an internal CA-signed TLS listener on
  `https://localhost:8443` with lightweight console logging.
- `overlays/staging.caddy` – Creates `staging.<domain>` with JSON logs
  that rotate more aggressively.
- `overlays/prod.caddy` – Adds `secure.<domain>` and enforces mTLS toward
  upstream workloads plus per-tenant rate limiting.

Launch an overlay by pointing Caddy at the file:

```bash
# Development example
source server/caddy/caddy.env
caddy run --config server/caddy/overlays/dev.caddy
```

## Portable Caddy install

The tooling mirrors the Cargo and Node installers already shipped in
`server/tools/`.

```bash
# Linux/macOS/WSL
./server/tools/setup-portable-caddy.sh
source ./server/tools/activate-caddy.sh

# Windows PowerShell
./server/tools/setup-portable-caddy.ps1
./server/tools/activate-caddy.ps1
```

The setup scripts place the binary under `server/tools/caddy-portable/`
and write `caddy-portable.manifest.json` so CI can audit the download.
Activation prepends the hermetic binary to `PATH` and exports
`NOA_CADDY_HOME`.

## CLI helpers

The Rust CLI exposes the admin API through the new `noa caddy` commands.
Examples:

```bash
# Push a new reverse proxy route
cargo run -p noa-cli -- caddy push-route \
  --domain api.noa-ark-os.local \
  --upstream localhost:8080 \
  --upstream localhost:8081 \
  --rate-limit-events 80

# Reload the running Caddy instance
cargo run -p noa-cli -- caddy reload
```

Both commands default to `http://127.0.0.1:2019` but accept custom admin
endpoints via `--admin-endpoint`.

## Certificate verification workflow

1. `source server/caddy/caddy.env` to load the admin email, domain, and
   storage root defaults.
2. Start Caddy via the base template or an overlay:
   `caddy run --config server/caddy/Caddyfile`.
3. For public domains, Let’s Encrypt certificates will be requested
   automatically. For local testing, run `caddy trust` to trust the
   internal CA and inspect the cert via
   `openssl s_client -connect localhost:8443 -servername localhost`.
4. Confirm the HTTP handler is healthy with
   `curl -k https://localhost:8443/health` or a domain-specific request.

## Observability, rate limiting, and logging

- **Rate limiting:** The base template and the prod overlay both declare
  `rate_limit` zones. Hit the endpoint repeatedly with `hey` or `curl`
  and look for `HTTP/1.1 429` responses plus `rate_limit` entries in the
  logs. Adjust thresholds via `NOA_CADDY_RATE_EVENTS` or CLI overrides.
- **Logging:** Access logs land in `logs/applications/caddy/*.log`. Tail
  them with `jq` or `bunyan` to confirm structured output. The staging
  and prod overlays emit JSON while the dev overlay prints in console
  format for easier local debugging.
- **Admin API telemetry:** Use `noa caddy reload` after editing
  `Caddyfile`. The command returns a JSON payload so automation can log
  reload events.

## HTTPS launch checklist

1. Install/activate the portable binary or ensure the system Caddy is in
   `PATH`.
2. Source `server/caddy/caddy.env` and optionally set
   `NOA_CADDY_PRIMARY_DOMAIN`, `NOA_CADDY_UPSTREAM`, or
   `NOA_CADDY_RATE_EVENTS`.
3. Run the desired configuration (base or overlay) with `caddy run`.
4. Use the CLI to push ad-hoc routes for experimental domains or
   microservices: `noa caddy push-route --domain demo.local --upstream
   localhost:18080`.
5. Reload via CLI or `caddy reload` whenever the config changes.
6. Verify HTTPS using `curl --cacert` or `openssl`, then inspect logs and
   rate-limit counters to prove the protections are active.
