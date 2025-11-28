# agentaskit SoT

<!--
State of Truth (SoT) ledger template.
Rules:
- Append-only for Executed Tasks; newest entries go to the end.
- Use UTC timestamps in ISO-like format: YYYY-MM-DD HH:MM UTC.
- Keep notes concise; link evidence and hashes.
- Avoid secrets/PII. Prefer relative repo paths.
-->

## 0) Meta
- Owner: <owner/team>
- Last updated: 2025-10-05 12:05 UTC
- Scope: agentaskit-production; repo-wide cross-reference & governance
- Status Note: Cross-reference CI added; artifacts generated per push.

## 1) Task Ledger

### 1.1 Executed Tasks (Chronological)
- [x] 2025-10-05 12:04 UTC — Archive Cross-Reference & Unification (WORKFLOW-006)
  - Artifacts:
    - agentaskit-production/docs/reports/cross_reference/artifacts/manifest.json
    - agentaskit-production/docs/reports/cross_reference/artifacts/report.json
    - agentaskit-production/docs/reports/cross_reference/artifacts/report.md
  - Notes: Added CI `.github/workflows/cross-reference.yml`, local hook `hooks/pre-push`, and scanner `tools/analysis/cross_reference.py`. Artifacts enumerate archive↔production lineage and missing components. Deterministic outputs.

<!--
- [x] <YYYY-MM-DD HH:MM UTC> — <Task title> — Artifacts: <path1>[, <path2> ...] — Notes: <what changed, why, how to reproduce in 1–2 sentences>

<!-- Optional detailed form
- [x] <YYYY-MM-DD HH:MM UTC> — <Task title>
  - Artifacts:
    - <relative/path/to/file_or_dir>
    - <relative/path/to/evidence_or_log>
  - Notes: <short description>
  - Repro:
    - Cmd: <exact command>
    - Output: <relative/path/to/transcript.log>
  - Hashes:
    - <relative/path/to/HASHES.txt#entry_or_checksum_ref>
-->

### 1.2 In-Progress Tasks
<!-- Track work with clear owners and deliverables. Convert to Executed when done. -->
- [ ] <YYYY-MM-DD HH:MM UTC> — <Workstream/Task name> — Owner: <name> — Status: <brief status>
  - Deliverables:
    - [ ] <deliverable 1>
    - [ ] <deliverable 2>
  - Artifacts (planned/current):
    - <relative/path/planned_or_wip>
  - Due: <YYYY-MM-DD>
  - Notes: <risks, blockers, decisions pending>

### 1.3 Planned / Backlog
- [ ] <Task name> — Rationale: <why> — Target: <YYYY-MM-DD> — Dependencies: <dep A, dep B>

## 2) Evidence, Repro, and Hashes
- Audit root: agentaskit-production/docs/reports/cross_reference/
- Repro commands ledger: agentaskit-production/docs/reports/cross_reference/README.md
- Test transcripts: agentaskit-production/TEST/*.log
- Hash manifests:
  - agentaskit-production/operational_hash/HASHES.txt
  - agentaskit-production/docs/reports/cross_reference/artifacts/manifest.json
- Standard procedure:
  1) Run tests/commands with deterministic flags (e.g., --no-color, fixed seeds).
  2) Save transcripts under TEST with UTC suffixes.
  3) Update commands ledger and hash manifests via: <script or command>.
  4) Append corresponding Executed Task entry with artifact paths and notes.

## 3) Governance & Standards
- Approvals/Acknowledgements:
  - <relative/path/to/governance/acknowledgements/*.md>
- Policies:
  - Development standards: <relative/path/to/standards.md>
  - Migration workflow: <relative/path/to/migration_workflow.md>
- Update flow:
  1) Draft change and evidence.
  2) Circulate for approval; capture acknowledgements.
  3) Refresh HASHES and append Executed Task entry.

## 4) Architecture & Integration Map
- Components:
  - Agents: <brief>
  - Services: <brief>
  - Frameworks: <brief>
  - Platform clients (desktop/mobile/web/xr): <brief>
  - Data/Docs/Tooling: <brief>
- External integrations:
  - <System/Integration A> — Interfaces: <CLI/HTTP/SDK> — Evidence: <path>
  - <System/Integration B> — Interfaces: <...> — Evidence: <path>
- Notes: <versioning, compatibility, constraints>

## 5) Risks, Decisions, and TODOs
- Decisions
  - <YYYY-MM-DD> — <decision made> — Owner: <name> — Context: <link/summary>
- Risks
  - <risk description> — Impact: <low/med/high> — Mitigation: <plan> — Owner: <name>
- TODOs
  - [ ] <action item> — Owner: <name> — Due: <YYYY-MM-DD>

## 6) Conventions
- Time: UTC, format YYYY-MM-DD HH:MM UTC.
- Paths: Relative to repo root.
- Hashing: sha256; store checksums in plain text manifests.
- Testing: Triple verification (PASS A/B/C) for stability-sensitive changes.
- Commits: Reference this SoT entry ID/timestamp in messages.

## 7) Quick Links
- Repo root: <./>
- CI workflows: <.github/workflows/>
- Services:
  - <services/service-a/>
  - <services/service-b/>
- Tooling:
  - Scripts: <tools/scripts/>
  - Bridges/Adapters: <tools/bridges/>

<!-- End of template -->