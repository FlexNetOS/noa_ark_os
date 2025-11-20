# Gateway Telemetry Storage

This directory holds OpenTelemetry-compatible artefacts emitted by the programmable gateway. The `noa_gateway` crate exports
structured metrics under `gateway_metrics.json` and appends span-aligned events to `gateway_events.log` so downstream collectors
can ingest them.

Files are generated automatically at runtime. It is safe to clean the directory between test runs.
