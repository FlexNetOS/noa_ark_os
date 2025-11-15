# Analytics Pipelines Storage

Notebook templates deposit summaries and metrics into this directory using
`ResearchBridge.push_summary()` and `ResearchBridge.push_metrics()`.

Each pipeline should write to a dedicated subdirectory (for example,
`storage/analytics/pipelines/digest-agent-metrics/summary.json`).
