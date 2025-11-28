-- Notebook Update Cadence Metric Model
-- Aggregates the average time between notebook commits and automation runs.
-- Source tables are exposed by the evidence lake under analytics.notebooks_* schemas.

WITH notebook_runs AS (
    SELECT
        run.notebook_id,
        run.workspace_id,
        run.executed_at,
        run.trigger_source,
        run.automation_scope
    FROM analytics.notebooks_run_log AS run
    WHERE run.executed_at >= DATEADD('day', -90, CURRENT_TIMESTAMP)
),
commit_deltas AS (
    SELECT
        runs.notebook_id,
        runs.workspace_id,
        runs.automation_scope,
        runs.trigger_source,
        runs.executed_at,
        LAG(runs.executed_at) OVER (PARTITION BY runs.notebook_id ORDER BY runs.executed_at) AS previous_run
    FROM notebook_runs AS runs
)
SELECT
    c.notebook_id,
    c.workspace_id,
    DATE_TRUNC('day', c.executed_at) AS activity_date,
    c.automation_scope,
    c.trigger_source,
    DATEDIFF('minute', c.previous_run, c.executed_at) AS minutes_between_runs,
    CASE
        WHEN c.previous_run IS NULL THEN NULL
        ELSE 1.0 / NULLIF(DATEDIFF('hour', c.previous_run, c.executed_at), 0)
    END AS cadence_per_hour,
    CURRENT_TIMESTAMP AS processed_at
FROM commit_deltas AS c;
