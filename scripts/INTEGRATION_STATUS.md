# Scripts → UI Integration Status

This ledger tracks which scripts in `./scripts` are exposed through the tooling catalog/UI and which still need wiring. Update it whenever a script gains frontend coverage or requires follow up.

| Script / Folder | Purpose | UI Tool ID | Status |
| --- | --- | --- | --- |
| `full_stack_launch.sh` | All-in-one workspace launcher (prep, kernel, Docker, services) | `full_stack_launch` | **Wired** – auto phases + new tool descriptor |
| `start-all-services.sh` | Legacy launcher for core services | _Pending_ | Needs consolidation behind UI tool; superseded by full stack launcher |
| `stop-all-services.sh` | Stops legacy service set | _Pending_ | Add UI action once legacy launcher is retired |
| `ultra-simple-start.ps1` | Minimal PowerShell bootstrap | _Pending_ | Requires Windows UI trigger |
| `dev/dev-env.sh` | Linux dev shell activation | _Pending_ | Should become a UI “Dev Shell” tool |
| `dev/dev-env.ps1` | Windows dev shell activation | _Pending_ | Needs PowerShell UI tool wiring |
| `codex-bootstrap.sh` | Tools-agent bootstrap helper | _Pending_ | Should register under tools-agent catalog |
| `guardrails-local.sh` | Local guardrails runner | _Pending_ | Connect to UI “safety” action |
| `pipeline/record_local_pipeline.sh` | Evidence capture helper | _Pending_ | Invoked indirectly today; expose standalone UI action |
| `automation/` | Holds automation DAG utilities | _Pending_ | Track per sub-script; not yet exposed |
| `integration/` | Integration runners/tests | _Pending_ | Needs tool definitions |
| `maintenance/` | Cleanup and archival routines | _Pending_ | Needs partitioned UI controls |
| `fix/` | Environment repair helpers | _Pending_ | Add curated UI buttons per fix |
| `setup/` | Host setup (CUDA, etc.) | _Partial_ | CUDA + llama setup now auto-launched; direct UI triggers still missing |
| `test/` | Automated test harness scripts | _Pending_ | Map into UI “Test” panel |

> _Legend_: **Wired** = available via tooling catalog/UI. _Partial_ = called indirectly but no dedicated UI entry. _Pending_ = not yet wired; see TODO backlog.

- 2025-11-18: `full_stack_launch` registered in `registry/tooling.catalog.json` with complete parameter coverage and CLI mapping.
