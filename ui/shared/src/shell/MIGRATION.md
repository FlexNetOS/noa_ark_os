# Unified shell migration guide

1. Replace surface-specific navigation with `navigationInventory` from `@noa-ark/shared-ui/shell`.
2. Instantiate `UnifiedShell` with your workflow endpoint and register the surfaces you intend to render.
3. Use `shell.componentFor(widget, surface)` to resolve shared components across desktop, web, and CLI renderers.
4. Adopt the `ShellStateManager` to obtain pluggable authentication providers and telemetry capture.
5. Export collected telemetry using `TelemetryClient.drain()` and feed it into the standard observability pipeline.
6. Gate experimental entry points behind the ecosystem feature toggles exposed in `services/marketplace`.
