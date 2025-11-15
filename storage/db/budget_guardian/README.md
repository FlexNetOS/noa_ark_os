# Budget Guardian Telemetry

This directory stores rolling summaries produced by the workflow `BudgetGuardian`.
Summaries are written to `rolling_summary.json` and include token and latency
percentiles derived from recent gateway telemetry events so Scorekeeper and
self-status surfaces can detect budget regressions offline.
