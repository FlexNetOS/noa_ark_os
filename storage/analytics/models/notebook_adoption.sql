-- Notebook Adoption Coverage Model
-- Captures notebook usage across workspaces and blueprints to highlight adoption trends.

WITH workspace_population AS (
    SELECT
        w.workspace_id,
        w.blueprint_id,
        w.owner_team,
        w.created_at
    FROM analytics.workspaces_dim AS w
),
notebook_usage AS (
    SELECT
        u.workspace_id,
        u.notebook_id,
        MIN(u.first_opened_at) AS first_opened_at,
        MAX(u.last_executed_at) AS last_executed_at,
        COUNT(DISTINCT u.user_id) AS unique_authors,
        SUM(u.automated_run_count) AS automated_run_count
    FROM analytics.notebooks_usage_fact AS u
    WHERE u.last_executed_at >= DATEADD('day', -90, CURRENT_TIMESTAMP)
    GROUP BY 1, 2
)
SELECT
    pop.workspace_id,
    pop.blueprint_id,
    pop.owner_team,
    usage.notebook_id,
    usage.unique_authors,
    usage.automated_run_count,
    CASE
        WHEN usage.first_opened_at IS NULL THEN 'inactive'
        WHEN usage.automated_run_count > 0 THEN 'automated'
        ELSE 'manual'
    END AS adoption_mode,
    DATE_TRUNC('week', COALESCE(usage.last_executed_at, pop.created_at)) AS active_week,
    CURRENT_TIMESTAMP AS processed_at
FROM workspace_population AS pop
LEFT JOIN notebook_usage AS usage
    ON usage.workspace_id = pop.workspace_id;
