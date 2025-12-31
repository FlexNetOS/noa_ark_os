# Reward Scorekeeper Overview

The scorekeeper monitors every workflow run and translates execution outcomes into weighted
rewards that incentivise healthy behaviour across the agent ecosystem. Each run produces a
`RewardDelta` entry that is appended to `metrics/reward_history.json` and surfaced through both
machine-readable and human-friendly reports.

## Inputs and Weights

| Metric            | Target / Expectation | Weight | Effect                               |
|-------------------|----------------------|--------|--------------------------------------|
| Test coverage     | ≥ 0.85               | +2.5   | Reward when coverage improves        |
| Flake rate        | ≤ 0.02               | +1.5   | Penalise flaky pipelines             |
| Token ratio       | ≤ 1.0 (budget)       | +1.0   | Penalise bloated prompts/execution   |
| Rollbacks         | 0 per run            | −1.25  | Each rollback subtracts from reward  |
| Failed agents     | —                    | −1.0   | Additional penalty for each failure  |

The scorekeeper automatically derives coverage and flake rate from the ratio of successful to
total agent tasks within a workflow run. Token ratio can be provided explicitly via task
parameters (e.g. `token_ratio`, `token_usage` / `token_budget`), and rollback penalties are
inferred from rollback actions or task metadata.

## History and Reporting

* **History ledger** – Each run appends a structured delta to `metrics/reward_history.json`.
  The file is stored as prettified JSON for easy inspection and downstream tooling.
* **CLI report** – `cargo run -p noa_workflow --bin reward_report` summarises trends,
  highlighting best-performing agents and any teams requiring manual approval. Use `--json`
  to emit machine-readable summaries (also available via `make scorekeeper`).
* **Metrics artefacts** – `make scorekeeper` produces:
  * `metrics/reward_summary.json` – aggregated trends in JSON form.
  * `metrics/reward_summary.txt` – human-readable dashboard snapshot.

## Integration with Workflow Approvals

The workflow engine queries the scorekeeper before dispatching any task. Agents whose cumulative
reward falls below −5.0 or whose recent trend is worse than −0.5 are flagged for manual
intervention. When such a flag is raised the workflow halts the task and records the failure,
preventing low-quality agents from degrading the system until their standing improves.

Developers can review outstanding flags with:

```bash
cargo run -p noa_workflow --bin reward_report
```

Repeated healthy runs automatically restore an agent’s standing by increasing the rolling
average, allowing the workflow to resume autonomous approvals once behaviour improves.

## Simulation & Tests

Unit tests in `workflow/src/reward.rs` simulate reward trajectories. They verify that:

* High flake rates, token excess and rollbacks trigger manual approval gates.
* Sustained improvements clear the penalties and allow automation to resume.

These tests exercise the same scoring logic used in production, ensuring the incentives align
with the “heal, don’t harm” policy.
