# Partner integration guidelines

1. Request licensing through the feature toggle manager before enabling partner bundles.
2. Validate that the gateway policy exposes the required ingress routes and telemetry sinks.
3. Ensure adaptive runtime assessments classify the host as `standard` or `accelerated` when GPU workloads are required.
4. Provide onboarding flows that reference the unified UI shell entry point IDs exported via the ecosystem bundle state file.
5. Register observability dashboards under `services/analytics/dashboards` to track adoption and ROI.
