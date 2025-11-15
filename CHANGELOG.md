# Changelog

## [Unreleased]

### Added

- `docs/guides/AGENTIC_OS_GUIDE.md` outlining operator workflows, policy
  enforcement, and CLI-first execution patterns.
- Kernel sovereignty and tokenized gateway diagrams under `.graphs/workflows/`
  with embedded copies in the new guide.
- Repository-wide markdown linting via `pnpm docs:lint` and `make docs:check`.

### Updated

- `AGENT.md` cross-links to the new guide and codifies Phase 0.5–10 patterns
  (snapshots, capability tokens, registry-only execution).
- `docs/guides/README.md` index now references the Agentic OS Operator Guide.
## Recent Gateway Changes (2025-11-15)

1edfa93a fix: update documentation and scripts to use corepack for pnpm commands
ed4e63fd Merge branch 'main' into codex/extend-evidence-show-command-with-filters
69e53284 feat(cli): extend evidence ledger inspection
50d02b33 Merge branch 'main' into codex/populate-telemetry-with-sample-metrics
dfb3acd3 Merge branch 'main' into codex/extend-cicd-service-to-log-pipeline-events
397e353c feat: add gateway telemetry fixtures and budget summaries
ccb26327 feat(cicd): emit pipeline state ledger events
38ce37e1 Merge branch 'main' into codex/add-storage-doctor-subcommand-with-tests
f6411e13 Merge branch 'main' into codex/create-folders-and-seed-artifacts-for-analytics
2a8d74f1 feat(analytics): materialize goal metrics and expose context rewards
2daa7e82 Add storage doctor CLI and CI guard
f1233358 Merge branch 'main' into codex/update-storage-readme-and-add-stubs
bc24372a Merge branch 'main' into codex/add-notebook-specific-metrics-and-dashboards
76b1d2d5 Merge branch 'main' into codex/add-research-notebook-package-and-publish-bundle
868252ca docs: document instrumentation mirrors
8d01add0 feat(storage): add notebook analytics telemetry
379ac3f4 feat: add research notebook workspace suite
65c037b3 Merge branch 'main' into copilot/merge-unify-forked-repos
b9fadb58 Merge branch 'main' into codex/add-cli-targets-for-cargo-and-pnpm
784219fd feat: add memory module to workspace members
d05dfb29 Merge branch 'main' into codex/enforce-pnpm-usage-and-update-ide-guidance
5db5de05 Merge branch 'main' into codex/create-machine-readable-policy-file
31934e58 Update docs/automation/command-policy-migration.md
f1c44e17 Merge branch 'main' into codex/create-reusable-dev-environment-provisioning-scripts
43ca4797 Update docs/guides/dev-environment-cli.md
d9972823 Merge branch 'main' into codex/implement-event-driven-triage-service
216744b3 Update docs/automation/triage_service.md
be34bcf2 feat: enhance agent dispatch and reward systems with improved metadata resolution and approval requirements
f104a9f8 feat: add WSL interoperability runbook and GitHub CLI authentication helper script
0b5b1cf8 Merge branch 'main' into codex/enhance-ci/cd-processes-and-documentation
f92343e6 Update docs/autonomous_release_flow.md
64ef0051 Merge branch 'main' into codex/implement-capability-token-service-and-tests
2b1f0fca Merge branch 'main' into codex/extend-scorekeeper-for-weighted-rewards
0f4f235e Merge branch 'main' into codex/implement-self-status-endpoint-with-enhancements
5aa11d80 Merge branch 'main' into codex/revise-documentation-for-new-cli-tooling
30c81400 fix: normalize vscode archive bundle encoding
55123871 feat(devshell): add portable make targets
6f43740c Merge branch 'main' into codex/implement-self-status-endpoint-with-enhancements
e515e922 Update docs/self_status_reflection.md
d36e868b Merge branch 'main' into codex/update-documentation-and-architecture-diagrams
1c486ebf Merge branch 'main' into codex/create-tools.registry.json-and-extend-noa-cli
bfb9e3ea Merge branch 'main' into codex/add-north_star.deflex.json-and-schema-validation
0a6dd190 Update docs/trust/trust_pipeline.md
8449e440 Update core/src/gateway.rs
4bd41773 Merge branch 'main' into codex/implement-real-http-integrations-and-observability
d40c1091 Merge branch 'main' into codex/add-profiles-and-extend-kernel-profile-loading
c854c488 feat: implement local-first pipeline evidence validation and GitHub workflow integration
44da70bc feat: add hermetic execution targets document outlining CLI-first and offline-first objectives
791608e6 feat: add devshell pnpm enforcement
79f336dc feat: centralize command safety policy
1ff3f4ee feat: add manifest-driven cli dev environment tooling
769980d8 fix: replace triage archive binary with text snapshot
da300d8a feat: harden autonomous release with trust ledger
ef25222d feat(core): add capability token service and host control harness
af7134fc feat(workflow): finalize reward instrumentation
7bbcabcb feat: expose self status reflection and automated repairs
e24c44e0 docs: add agentic operator guide and lint checks
dca6a771 feat(cli): add registry-driven tooling surfaces
00af3946 fix(workflow): harden trust guardrails
9bc3ae02 Implement streaming providers with telemetry and failover
73b92772 Merge branch 'main' into codex/implement-sbom-and-audit-pipeline
87dbe56c Merge branch 'main' into codex/implement-world-graph-and-reconciler-logic
e46d863b Merge branch 'main' into codex/implement-quarantine-archiving-and-ci-checks
4f8e8f36 feat: add quarantine governance tooling
05e2b26d feat(core): issue profile-scoped capability tokens
8b11c7c6 feat: add reproducible audit pipeline tooling
b6f08a53 feat(kernel): add world graph reconciler and tests
96dc3ecf Merge branch 'main' into codex/enhance-code-for-production-readiness
9f1ad4c8 Update docs/tasks/ROADMAP_AGENTIC_KERNEL_TASK_LINKS.md
02ca54e6 Merge branch 'main' into codex/implement-agent-managed-pipelines-and-automation
9e983dd6 Expand Agentic Kernel task launch catalog
27a5a998 Merge pull request #177 from FlexNetOS/codex/update-inventory-role-references-and-sops
a33e641e Merge pull request #178 from FlexNetOS/codex/implement-automated-training-orchestration-in-ai
73d0346b fix: finalize agent managed workflow pipelines
27cc82e9 Merge remote-tracking branch 'origin/main'
d12f2c20 feat(ai): add training orchestrator and ml ops controls
7ae40a28 docs: align operational guides with agent hierarchy
f7f9db3b docs: refresh strategic roadmaps
889f6781 Merge branch 'main' into codex/add-new-capability-tokens-and-analytics-features
b8231d62 Merge branch 'main' into codex/expand-ai-database-with-new-tables-and-apis
6fd68c33 Merge branch 'feat/ci-guardrails-taskspec' into main
b705a8b1 feat: wire legacy feature flag into Cargo.toml and add LEGACY_INVENTORY.md scaffold
cb7454cc feat: add kanban goal insights analytics and autonomy
e8d5628d feat: persist workspace goal memory and planner insights
b4f2fed6 feat(ui): migrate vibe kanban to goal model
892c2005 feat(ui-api): add /api/uploads alias and /api/capabilities endpoint feat(orchestrator): support glob artifacts and maintain docs/tasks/index.json chore: reset docs/tasks/index.json placeholder
f7c8ba3c Merge branch 'main' into feat/ci-guardrails-taskspec
0e567bb7 feat(ui-api): wire Upload→Digest endpoint tests; keep receipts under crc/drop-in/incoming/receipts chore(workflow): TaskSpec acceptance uses noa_crc + noa_ui_api tests and ingest report ci: add Smoke Suite workflow to run CAS, Digest, Upload→Digest
c4291918 feat: stabilize instrumentation and wasm integrations
d886151b feat(ci): guardrails (deletion-lock, duplicate-fence, report-fence, change-budget) chore(ci): policy conformance (AGENT.md pointers) + single TS router enforcement feat(workflow): TaskSpec schema + validator; seed example and evidence ledger chore(staging): prior_projects lane skeleton
bf5c84d7 Merge branch 'main' into codex/archive-bespoke-router/controller-copies
4e9e9e66 Replace archived router snapshots with text copies
6dbd3c5e Merge pull request #155 from FlexNetOS/codex/create-documented-tasks-index-and-validation
899b86eb feat: add roadmap task index and validator
97d6ed28 feat(ui): add capability registry API and gating
d2cc0cdb Handle Ubuntu ESM .sources files in firewall fix script
bfdc7dbb Merge pull request #115 from FlexNetOS/codex/create-gh-issue-for-agentos-1-task
f4e706a7 Update docs/plans/gap_remediation_tasks.md
ba9041f0 Merge branch 'main' into codex/create-issue-agentos-4-for-adaptive-orchestration
4a74f788 Merge branch 'main' into codex/create-issue-for-advanced-gateway-development
e18fdd0e Enrich AGENTOS-5 suggested task links
7e891551 Link AGENTOS-4 issue to suggested tasks
0e086bab Merge branch 'main' into codex/file-issue-agentos-3-for-ui-shell
6702ddb9 Merge pull request #118 from FlexNetOS/codex/update-agentos-4-documentation-tasks-list
30125d31 Add internal issue stubs for remaining gap remediation tasks
ee97af5a Link AGENTOS-4 to specific gap remediation tasks
6cc72730 Merge pull request #117 from FlexNetOS/codex/create-issue-agentos-2-for-agentos-server-profile
52580505 docs: add suggested task links
6afc23ca Merge pull request #116 from FlexNetOS/codex/create-gh-issue-for-agentos-1-task-5mdy5d
55882305 Deduplicate gap remediation task anchors
4fdcf1dc Deduplicate gap remediation task anchors
5c1bb827 Merge branch 'main' into codex/update-automation-master-plan-documentation
daf8013c Add automation master plan HITL coverage and tasks
160a72c9 docs: add suggested task links for automation plan
bd10570f Merge branch 'main' into codex/add-ai-assisted-task-prompts-to-project-management-app
f7e64095 feat: add ai assist prompt flow and roadmap exports
15691e1c Merge branch 'main' into codex/add-gap-remediation-tasks-section-in-markdown
474103c9 chore: script GitHub issue creation for agentos tasks
b51bfdd4 Merge pull request #94 from FlexNetOS/codex/edit-agentos-task-details-in-documentation
ce432a52 Update docs/plans/gap_remediation_tasks.md
82f9e146 Update docs/plans/gap_remediation_tasks.md
b7c52a99 Merge branch 'main' into codex/create-github-issues-for-agentos-tasks
b1b99be8 Track AGENTOS remediation tasks in issues
3291d7f8 Merge branch 'main' into codex/edit-agentos-task-details-in-documentation
f388e515 Add AGENTOS task overview anchors
9501cb97 Merge branch 'main' into codex/add-gap-remediation-tasks-section-in-markdown
6a0b9ad5 Add GitHub CLI installation helper
e550a117 Merge pull request #89 from FlexNetOS/codex/file-issue-for-agentos-6-features
cdbed298 Update docs/issues/AGENTOS-6.md
c2c6edc7 Update docs/issues/AGENTOS-6.md
69e251dc Update docs/issues/AGENTOS-6.md
81ed6157 Merge pull request #92 from FlexNetOS/codex/file-issue-agentos-3-for-ui-shell
02e1836c Merge pull request #91 from FlexNetOS/codex/create-issue-agentos-4-for-adaptive-orchestration
b0cf9d1c Update docs/issues/AGENTOS-4.md
2be30524 Merge pull request #90 from FlexNetOS/codex/create-issue-for-advanced-gateway-development
8d11ff73 Document AGENTOS-3 unified UI shell issue
3b31f97d Register AGENTOS-4 adaptive runtime orchestration issue
16600429 Add AGENTOS-5 advanced gateway issue summary
4d7b501b Add AGENTOS-6 issue for ecosystem launch
cb9ec6bb Merge pull request #88 from FlexNetOS/codex/add-gap-remediation-tasks-section-in-markdown
e6420d10 Fix AGENTOS-1 task title wording
3b4524cf Merge pull request #86 from FlexNetOS/codex/fix-firewall-rules-blocking-connections
a106ed67 Add script to disable Ubuntu ESM apt sources
c78b250c Restructure gap remediation tasks with objectives
44e10759 Add remediation task list for outstanding roadmap gaps
cf562579 Merge branch 'main' into codex/instrument-relocation-and-doc-pipelines
b4b504b1 Stabilize policy signatures and ledger handling
81d43a15 Merge branch 'main' into codex/model-service-dependencies-as-kernel-managed-capabilities
55882f48 Add kernel capability registry and portable tooling
5201614c Merge branch 'main' into codex/extend-configuration-with-automation-playbooks
34c38838 Refine automation telemetry configs and reporting
825232ba Update server/gateway/src/telemetry.rs
a2000cce Update server/gateway/src/auth.rs
b594b75b Merge branch 'main' into codex/expand-server-gateway-functionalities-eevpx3
14e648ce Tighten gateway auth, telemetry, and tests
4dbf0f0b Merge branch 'main' into codex/generate-documentation-infrastructure-and-workflows
1585a20e Revert "Add programmable gateway infrastructure"
07ea721c Merge branch 'main' into codex/expand-server-gateway-functionalities
53542ab3 Merge pull request #36 from FlexNetOS/codex/define-shared-design-system
be77d473 Merge branch 'main' into codex/implement-consolidated-schema-for-registry
6afe6cfe Add documentation sync agent and doc refresh pipeline
088fed0f Build registry knowledge graph service and tooling
52f04e26 Merge branch 'main' into codex/introduce-hardware-capability-detection
dea357e6 Merge pull request #35 from FlexNetOS/codex/implement-single-host-profile-configuration
a4ae6b24 Add programmable gateway orchestration
7f18eb66 Add hardware detection and runtime backend selection
5391c222 feat: add shared ui design system and streaming schema
219b837f feat: add single-host deployment profile
402dd310 Update docs/community/marketplace-contributions.md
dbe15cb8 Add blueprint, marketplace, analytics, and community docs
e2718cff Merge pull request #23 from FlexNetOS/codex/complete-phase-6-of-ui/ux-roadmap-a3kxl0
57c6f057 Update core/src/gateway.rs
23130def Update core/src/gateway.rs
368a64e2 Update core/src/gateway.rs
1c496b29 Update core/src/gateway.rs
5d06dd4c Clean gateway orchestration definitions
9d5b809a Finalize gateway automation fabric validation
414a102c Merge branch 'main' into codex/complete-phases-0-3-health-check-and-upgrade-code
04e32a20 Unify shell modules and expand unified API surface
3e1ce9fd Merge branch 'main' into codex/enhance-design-for-universal-gateway-u8kiu4
d6fce8fd Implement zero-trust routing intelligence for the gateway
b40b8288 Merge branch 'main' into codex/add-gateway-upgrades-and-features
f9375634 Enhance gateway with adaptive verification and governance
d594096d Merge pull request #13 from FlexNetOS/codex/add-upgrades-to-gateway-architecture
a12385d7 feat: upgrade gateway architecture
5ae273a1 Merge branch 'main' into codex/create-roadmap-for-integrated-ui-features-pcoj0l
1a2d04ae Implement persona-aware adapters and knowledge overlay
1e37c468 Merge branch 'main' into codex/create-roadmap-for-integrated-ui-features-gku3wk
b00b12ea Revamp unified control plane UI and API
dc20388b Merge branch 'main' into codex/enhance-design-for-universal-gateway-06p375
25030b2f Restore gitattributes and add WSL cargo activator
eb269432 Add gateway roadmap plan
ed4e5b69 Refine gateway orchestration and safety

## Recent Gateway Changes (2025-11-15)

1edfa93a fix: update documentation and scripts to use corepack for pnpm commands
ed4e63fd Merge branch 'main' into codex/extend-evidence-show-command-with-filters
69e53284 feat(cli): extend evidence ledger inspection
50d02b33 Merge branch 'main' into codex/populate-telemetry-with-sample-metrics
dfb3acd3 Merge branch 'main' into codex/extend-cicd-service-to-log-pipeline-events
397e353c feat: add gateway telemetry fixtures and budget summaries
ccb26327 feat(cicd): emit pipeline state ledger events
38ce37e1 Merge branch 'main' into codex/add-storage-doctor-subcommand-with-tests
f6411e13 Merge branch 'main' into codex/create-folders-and-seed-artifacts-for-analytics
2a8d74f1 feat(analytics): materialize goal metrics and expose context rewards
2daa7e82 Add storage doctor CLI and CI guard
f1233358 Merge branch 'main' into codex/update-storage-readme-and-add-stubs
bc24372a Merge branch 'main' into codex/add-notebook-specific-metrics-and-dashboards
76b1d2d5 Merge branch 'main' into codex/add-research-notebook-package-and-publish-bundle
868252ca docs: document instrumentation mirrors
8d01add0 feat(storage): add notebook analytics telemetry
379ac3f4 feat: add research notebook workspace suite
65c037b3 Merge branch 'main' into copilot/merge-unify-forked-repos
b9fadb58 Merge branch 'main' into codex/add-cli-targets-for-cargo-and-pnpm
784219fd feat: add memory module to workspace members
d05dfb29 Merge branch 'main' into codex/enforce-pnpm-usage-and-update-ide-guidance
5db5de05 Merge branch 'main' into codex/create-machine-readable-policy-file
31934e58 Update docs/automation/command-policy-migration.md
f1c44e17 Merge branch 'main' into codex/create-reusable-dev-environment-provisioning-scripts
43ca4797 Update docs/guides/dev-environment-cli.md
d9972823 Merge branch 'main' into codex/implement-event-driven-triage-service
216744b3 Update docs/automation/triage_service.md
be34bcf2 feat: enhance agent dispatch and reward systems with improved metadata resolution and approval requirements
f104a9f8 feat: add WSL interoperability runbook and GitHub CLI authentication helper script
0b5b1cf8 Merge branch 'main' into codex/enhance-ci/cd-processes-and-documentation
f92343e6 Update docs/autonomous_release_flow.md
64ef0051 Merge branch 'main' into codex/implement-capability-token-service-and-tests
2b1f0fca Merge branch 'main' into codex/extend-scorekeeper-for-weighted-rewards
0f4f235e Merge branch 'main' into codex/implement-self-status-endpoint-with-enhancements
5aa11d80 Merge branch 'main' into codex/revise-documentation-for-new-cli-tooling
30c81400 fix: normalize vscode archive bundle encoding
55123871 feat(devshell): add portable make targets
6f43740c Merge branch 'main' into codex/implement-self-status-endpoint-with-enhancements
e515e922 Update docs/self_status_reflection.md
d36e868b Merge branch 'main' into codex/update-documentation-and-architecture-diagrams
1c486ebf Merge branch 'main' into codex/create-tools.registry.json-and-extend-noa-cli
bfb9e3ea Merge branch 'main' into codex/add-north_star.deflex.json-and-schema-validation
0a6dd190 Update docs/trust/trust_pipeline.md
8449e440 Update core/src/gateway.rs
4bd41773 Merge branch 'main' into codex/implement-real-http-integrations-and-observability
d40c1091 Merge branch 'main' into codex/add-profiles-and-extend-kernel-profile-loading
c854c488 feat: implement local-first pipeline evidence validation and GitHub workflow integration
44da70bc feat: add hermetic execution targets document outlining CLI-first and offline-first objectives
791608e6 feat: add devshell pnpm enforcement
79f336dc feat: centralize command safety policy
1ff3f4ee feat: add manifest-driven cli dev environment tooling
769980d8 fix: replace triage archive binary with text snapshot
da300d8a feat: harden autonomous release with trust ledger
ef25222d feat(core): add capability token service and host control harness
af7134fc feat(workflow): finalize reward instrumentation
7bbcabcb feat: expose self status reflection and automated repairs
e24c44e0 docs: add agentic operator guide and lint checks
dca6a771 feat(cli): add registry-driven tooling surfaces
00af3946 fix(workflow): harden trust guardrails
9bc3ae02 Implement streaming providers with telemetry and failover
73b92772 Merge branch 'main' into codex/implement-sbom-and-audit-pipeline
87dbe56c Merge branch 'main' into codex/implement-world-graph-and-reconciler-logic
e46d863b Merge branch 'main' into codex/implement-quarantine-archiving-and-ci-checks
4f8e8f36 feat: add quarantine governance tooling
05e2b26d feat(core): issue profile-scoped capability tokens
8b11c7c6 feat: add reproducible audit pipeline tooling
b6f08a53 feat(kernel): add world graph reconciler and tests
96dc3ecf Merge branch 'main' into codex/enhance-code-for-production-readiness
9f1ad4c8 Update docs/tasks/ROADMAP_AGENTIC_KERNEL_TASK_LINKS.md
02ca54e6 Merge branch 'main' into codex/implement-agent-managed-pipelines-and-automation
9e983dd6 Expand Agentic Kernel task launch catalog
27a5a998 Merge pull request #177 from FlexNetOS/codex/update-inventory-role-references-and-sops
a33e641e Merge pull request #178 from FlexNetOS/codex/implement-automated-training-orchestration-in-ai
73d0346b fix: finalize agent managed workflow pipelines
27cc82e9 Merge remote-tracking branch 'origin/main'
d12f2c20 feat(ai): add training orchestrator and ml ops controls
7ae40a28 docs: align operational guides with agent hierarchy
f7f9db3b docs: refresh strategic roadmaps
889f6781 Merge branch 'main' into codex/add-new-capability-tokens-and-analytics-features
b8231d62 Merge branch 'main' into codex/expand-ai-database-with-new-tables-and-apis
6fd68c33 Merge branch 'feat/ci-guardrails-taskspec' into main
b705a8b1 feat: wire legacy feature flag into Cargo.toml and add LEGACY_INVENTORY.md scaffold
cb7454cc feat: add kanban goal insights analytics and autonomy
e8d5628d feat: persist workspace goal memory and planner insights
b4f2fed6 feat(ui): migrate vibe kanban to goal model
892c2005 feat(ui-api): add /api/uploads alias and /api/capabilities endpoint feat(orchestrator): support glob artifacts and maintain docs/tasks/index.json chore: reset docs/tasks/index.json placeholder
f7c8ba3c Merge branch 'main' into feat/ci-guardrails-taskspec
0e567bb7 feat(ui-api): wire Upload→Digest endpoint tests; keep receipts under crc/drop-in/incoming/receipts chore(workflow): TaskSpec acceptance uses noa_crc + noa_ui_api tests and ingest report ci: add Smoke Suite workflow to run CAS, Digest, Upload→Digest
c4291918 feat: stabilize instrumentation and wasm integrations
d886151b feat(ci): guardrails (deletion-lock, duplicate-fence, report-fence, change-budget) chore(ci): policy conformance (AGENT.md pointers) + single TS router enforcement feat(workflow): TaskSpec schema + validator; seed example and evidence ledger chore(staging): prior_projects lane skeleton
bf5c84d7 Merge branch 'main' into codex/archive-bespoke-router/controller-copies
4e9e9e66 Replace archived router snapshots with text copies
6dbd3c5e Merge pull request #155 from FlexNetOS/codex/create-documented-tasks-index-and-validation
899b86eb feat: add roadmap task index and validator
97d6ed28 feat(ui): add capability registry API and gating
d2cc0cdb Handle Ubuntu ESM .sources files in firewall fix script
bfdc7dbb Merge pull request #115 from FlexNetOS/codex/create-gh-issue-for-agentos-1-task
f4e706a7 Update docs/plans/gap_remediation_tasks.md
ba9041f0 Merge branch 'main' into codex/create-issue-agentos-4-for-adaptive-orchestration
4a74f788 Merge branch 'main' into codex/create-issue-for-advanced-gateway-development
e18fdd0e Enrich AGENTOS-5 suggested task links
7e891551 Link AGENTOS-4 issue to suggested tasks
0e086bab Merge branch 'main' into codex/file-issue-agentos-3-for-ui-shell
6702ddb9 Merge pull request #118 from FlexNetOS/codex/update-agentos-4-documentation-tasks-list
30125d31 Add internal issue stubs for remaining gap remediation tasks
ee97af5a Link AGENTOS-4 to specific gap remediation tasks
6cc72730 Merge pull request #117 from FlexNetOS/codex/create-issue-agentos-2-for-agentos-server-profile
52580505 docs: add suggested task links
6afc23ca Merge pull request #116 from FlexNetOS/codex/create-gh-issue-for-agentos-1-task-5mdy5d
55882305 Deduplicate gap remediation task anchors
4fdcf1dc Deduplicate gap remediation task anchors
5c1bb827 Merge branch 'main' into codex/update-automation-master-plan-documentation
daf8013c Add automation master plan HITL coverage and tasks
160a72c9 docs: add suggested task links for automation plan
bd10570f Merge branch 'main' into codex/add-ai-assisted-task-prompts-to-project-management-app
f7e64095 feat: add ai assist prompt flow and roadmap exports
15691e1c Merge branch 'main' into codex/add-gap-remediation-tasks-section-in-markdown
474103c9 chore: script GitHub issue creation for agentos tasks
b51bfdd4 Merge pull request #94 from FlexNetOS/codex/edit-agentos-task-details-in-documentation
ce432a52 Update docs/plans/gap_remediation_tasks.md
82f9e146 Update docs/plans/gap_remediation_tasks.md
b7c52a99 Merge branch 'main' into codex/create-github-issues-for-agentos-tasks
b1b99be8 Track AGENTOS remediation tasks in issues
3291d7f8 Merge branch 'main' into codex/edit-agentos-task-details-in-documentation
f388e515 Add AGENTOS task overview anchors
9501cb97 Merge branch 'main' into codex/add-gap-remediation-tasks-section-in-markdown
6a0b9ad5 Add GitHub CLI installation helper
e550a117 Merge pull request #89 from FlexNetOS/codex/file-issue-for-agentos-6-features
cdbed298 Update docs/issues/AGENTOS-6.md
c2c6edc7 Update docs/issues/AGENTOS-6.md
69e251dc Update docs/issues/AGENTOS-6.md
81ed6157 Merge pull request #92 from FlexNetOS/codex/file-issue-agentos-3-for-ui-shell
02e1836c Merge pull request #91 from FlexNetOS/codex/create-issue-agentos-4-for-adaptive-orchestration
b0cf9d1c Update docs/issues/AGENTOS-4.md
2be30524 Merge pull request #90 from FlexNetOS/codex/create-issue-for-advanced-gateway-development
8d11ff73 Document AGENTOS-3 unified UI shell issue
3b31f97d Register AGENTOS-4 adaptive runtime orchestration issue
16600429 Add AGENTOS-5 advanced gateway issue summary
4d7b501b Add AGENTOS-6 issue for ecosystem launch
cb9ec6bb Merge pull request #88 from FlexNetOS/codex/add-gap-remediation-tasks-section-in-markdown
e6420d10 Fix AGENTOS-1 task title wording
3b4524cf Merge pull request #86 from FlexNetOS/codex/fix-firewall-rules-blocking-connections
a106ed67 Add script to disable Ubuntu ESM apt sources
c78b250c Restructure gap remediation tasks with objectives
44e10759 Add remediation task list for outstanding roadmap gaps
cf562579 Merge branch 'main' into codex/instrument-relocation-and-doc-pipelines
b4b504b1 Stabilize policy signatures and ledger handling
81d43a15 Merge branch 'main' into codex/model-service-dependencies-as-kernel-managed-capabilities
55882f48 Add kernel capability registry and portable tooling
5201614c Merge branch 'main' into codex/extend-configuration-with-automation-playbooks
34c38838 Refine automation telemetry configs and reporting
825232ba Update server/gateway/src/telemetry.rs
a2000cce Update server/gateway/src/auth.rs
b594b75b Merge branch 'main' into codex/expand-server-gateway-functionalities-eevpx3
14e648ce Tighten gateway auth, telemetry, and tests
4dbf0f0b Merge branch 'main' into codex/generate-documentation-infrastructure-and-workflows
1585a20e Revert "Add programmable gateway infrastructure"
07ea721c Merge branch 'main' into codex/expand-server-gateway-functionalities
53542ab3 Merge pull request #36 from FlexNetOS/codex/define-shared-design-system
be77d473 Merge branch 'main' into codex/implement-consolidated-schema-for-registry
6afe6cfe Add documentation sync agent and doc refresh pipeline
088fed0f Build registry knowledge graph service and tooling
52f04e26 Merge branch 'main' into codex/introduce-hardware-capability-detection
dea357e6 Merge pull request #35 from FlexNetOS/codex/implement-single-host-profile-configuration
a4ae6b24 Add programmable gateway orchestration
7f18eb66 Add hardware detection and runtime backend selection
5391c222 feat: add shared ui design system and streaming schema
219b837f feat: add single-host deployment profile
402dd310 Update docs/community/marketplace-contributions.md
dbe15cb8 Add blueprint, marketplace, analytics, and community docs
e2718cff Merge pull request #23 from FlexNetOS/codex/complete-phase-6-of-ui/ux-roadmap-a3kxl0
57c6f057 Update core/src/gateway.rs
23130def Update core/src/gateway.rs
368a64e2 Update core/src/gateway.rs
1c496b29 Update core/src/gateway.rs
5d06dd4c Clean gateway orchestration definitions
9d5b809a Finalize gateway automation fabric validation
414a102c Merge branch 'main' into codex/complete-phases-0-3-health-check-and-upgrade-code
04e32a20 Unify shell modules and expand unified API surface
3e1ce9fd Merge branch 'main' into codex/enhance-design-for-universal-gateway-u8kiu4
d6fce8fd Implement zero-trust routing intelligence for the gateway
b40b8288 Merge branch 'main' into codex/add-gateway-upgrades-and-features
f9375634 Enhance gateway with adaptive verification and governance
d594096d Merge pull request #13 from FlexNetOS/codex/add-upgrades-to-gateway-architecture
a12385d7 feat: upgrade gateway architecture
5ae273a1 Merge branch 'main' into codex/create-roadmap-for-integrated-ui-features-pcoj0l
1a2d04ae Implement persona-aware adapters and knowledge overlay
1e37c468 Merge branch 'main' into codex/create-roadmap-for-integrated-ui-features-gku3wk
b00b12ea Revamp unified control plane UI and API
dc20388b Merge branch 'main' into codex/enhance-design-for-universal-gateway-06p375
25030b2f Restore gitattributes and add WSL cargo activator
eb269432 Add gateway roadmap plan
ed4e5b69 Refine gateway orchestration and safety

## Recent Gateway Changes (2025-11-15)

1edfa93a fix: update documentation and scripts to use corepack for pnpm commands
ed4e63fd Merge branch 'main' into codex/extend-evidence-show-command-with-filters
69e53284 feat(cli): extend evidence ledger inspection
50d02b33 Merge branch 'main' into codex/populate-telemetry-with-sample-metrics
dfb3acd3 Merge branch 'main' into codex/extend-cicd-service-to-log-pipeline-events
397e353c feat: add gateway telemetry fixtures and budget summaries
ccb26327 feat(cicd): emit pipeline state ledger events
38ce37e1 Merge branch 'main' into codex/add-storage-doctor-subcommand-with-tests
f6411e13 Merge branch 'main' into codex/create-folders-and-seed-artifacts-for-analytics
2a8d74f1 feat(analytics): materialize goal metrics and expose context rewards
2daa7e82 Add storage doctor CLI and CI guard
f1233358 Merge branch 'main' into codex/update-storage-readme-and-add-stubs
bc24372a Merge branch 'main' into codex/add-notebook-specific-metrics-and-dashboards
76b1d2d5 Merge branch 'main' into codex/add-research-notebook-package-and-publish-bundle
868252ca docs: document instrumentation mirrors
8d01add0 feat(storage): add notebook analytics telemetry
379ac3f4 feat: add research notebook workspace suite
65c037b3 Merge branch 'main' into copilot/merge-unify-forked-repos
b9fadb58 Merge branch 'main' into codex/add-cli-targets-for-cargo-and-pnpm
784219fd feat: add memory module to workspace members
d05dfb29 Merge branch 'main' into codex/enforce-pnpm-usage-and-update-ide-guidance
5db5de05 Merge branch 'main' into codex/create-machine-readable-policy-file
31934e58 Update docs/automation/command-policy-migration.md
f1c44e17 Merge branch 'main' into codex/create-reusable-dev-environment-provisioning-scripts
43ca4797 Update docs/guides/dev-environment-cli.md
d9972823 Merge branch 'main' into codex/implement-event-driven-triage-service
216744b3 Update docs/automation/triage_service.md
be34bcf2 feat: enhance agent dispatch and reward systems with improved metadata resolution and approval requirements
f104a9f8 feat: add WSL interoperability runbook and GitHub CLI authentication helper script
0b5b1cf8 Merge branch 'main' into codex/enhance-ci/cd-processes-and-documentation
f92343e6 Update docs/autonomous_release_flow.md
64ef0051 Merge branch 'main' into codex/implement-capability-token-service-and-tests
2b1f0fca Merge branch 'main' into codex/extend-scorekeeper-for-weighted-rewards
0f4f235e Merge branch 'main' into codex/implement-self-status-endpoint-with-enhancements
5aa11d80 Merge branch 'main' into codex/revise-documentation-for-new-cli-tooling
30c81400 fix: normalize vscode archive bundle encoding
55123871 feat(devshell): add portable make targets
6f43740c Merge branch 'main' into codex/implement-self-status-endpoint-with-enhancements
e515e922 Update docs/self_status_reflection.md
d36e868b Merge branch 'main' into codex/update-documentation-and-architecture-diagrams
1c486ebf Merge branch 'main' into codex/create-tools.registry.json-and-extend-noa-cli
bfb9e3ea Merge branch 'main' into codex/add-north_star.deflex.json-and-schema-validation
0a6dd190 Update docs/trust/trust_pipeline.md
8449e440 Update core/src/gateway.rs
4bd41773 Merge branch 'main' into codex/implement-real-http-integrations-and-observability
d40c1091 Merge branch 'main' into codex/add-profiles-and-extend-kernel-profile-loading
c854c488 feat: implement local-first pipeline evidence validation and GitHub workflow integration
44da70bc feat: add hermetic execution targets document outlining CLI-first and offline-first objectives
791608e6 feat: add devshell pnpm enforcement
79f336dc feat: centralize command safety policy
1ff3f4ee feat: add manifest-driven cli dev environment tooling
769980d8 fix: replace triage archive binary with text snapshot
da300d8a feat: harden autonomous release with trust ledger
ef25222d feat(core): add capability token service and host control harness
af7134fc feat(workflow): finalize reward instrumentation
7bbcabcb feat: expose self status reflection and automated repairs
e24c44e0 docs: add agentic operator guide and lint checks
dca6a771 feat(cli): add registry-driven tooling surfaces
00af3946 fix(workflow): harden trust guardrails
9bc3ae02 Implement streaming providers with telemetry and failover
73b92772 Merge branch 'main' into codex/implement-sbom-and-audit-pipeline
87dbe56c Merge branch 'main' into codex/implement-world-graph-and-reconciler-logic
e46d863b Merge branch 'main' into codex/implement-quarantine-archiving-and-ci-checks
4f8e8f36 feat: add quarantine governance tooling
05e2b26d feat(core): issue profile-scoped capability tokens
8b11c7c6 feat: add reproducible audit pipeline tooling
b6f08a53 feat(kernel): add world graph reconciler and tests
96dc3ecf Merge branch 'main' into codex/enhance-code-for-production-readiness
9f1ad4c8 Update docs/tasks/ROADMAP_AGENTIC_KERNEL_TASK_LINKS.md
02ca54e6 Merge branch 'main' into codex/implement-agent-managed-pipelines-and-automation
9e983dd6 Expand Agentic Kernel task launch catalog
27a5a998 Merge pull request #177 from FlexNetOS/codex/update-inventory-role-references-and-sops
a33e641e Merge pull request #178 from FlexNetOS/codex/implement-automated-training-orchestration-in-ai
73d0346b fix: finalize agent managed workflow pipelines
27cc82e9 Merge remote-tracking branch 'origin/main'
d12f2c20 feat(ai): add training orchestrator and ml ops controls
7ae40a28 docs: align operational guides with agent hierarchy
f7f9db3b docs: refresh strategic roadmaps
889f6781 Merge branch 'main' into codex/add-new-capability-tokens-and-analytics-features
b8231d62 Merge branch 'main' into codex/expand-ai-database-with-new-tables-and-apis
6fd68c33 Merge branch 'feat/ci-guardrails-taskspec' into main
b705a8b1 feat: wire legacy feature flag into Cargo.toml and add LEGACY_INVENTORY.md scaffold
cb7454cc feat: add kanban goal insights analytics and autonomy
e8d5628d feat: persist workspace goal memory and planner insights
b4f2fed6 feat(ui): migrate vibe kanban to goal model
892c2005 feat(ui-api): add /api/uploads alias and /api/capabilities endpoint feat(orchestrator): support glob artifacts and maintain docs/tasks/index.json chore: reset docs/tasks/index.json placeholder
f7c8ba3c Merge branch 'main' into feat/ci-guardrails-taskspec
0e567bb7 feat(ui-api): wire Upload→Digest endpoint tests; keep receipts under crc/drop-in/incoming/receipts chore(workflow): TaskSpec acceptance uses noa_crc + noa_ui_api tests and ingest report ci: add Smoke Suite workflow to run CAS, Digest, Upload→Digest
c4291918 feat: stabilize instrumentation and wasm integrations
d886151b feat(ci): guardrails (deletion-lock, duplicate-fence, report-fence, change-budget) chore(ci): policy conformance (AGENT.md pointers) + single TS router enforcement feat(workflow): TaskSpec schema + validator; seed example and evidence ledger chore(staging): prior_projects lane skeleton
bf5c84d7 Merge branch 'main' into codex/archive-bespoke-router/controller-copies
4e9e9e66 Replace archived router snapshots with text copies
6dbd3c5e Merge pull request #155 from FlexNetOS/codex/create-documented-tasks-index-and-validation
899b86eb feat: add roadmap task index and validator
97d6ed28 feat(ui): add capability registry API and gating
d2cc0cdb Handle Ubuntu ESM .sources files in firewall fix script
bfdc7dbb Merge pull request #115 from FlexNetOS/codex/create-gh-issue-for-agentos-1-task
f4e706a7 Update docs/plans/gap_remediation_tasks.md
ba9041f0 Merge branch 'main' into codex/create-issue-agentos-4-for-adaptive-orchestration
4a74f788 Merge branch 'main' into codex/create-issue-for-advanced-gateway-development
e18fdd0e Enrich AGENTOS-5 suggested task links
7e891551 Link AGENTOS-4 issue to suggested tasks
0e086bab Merge branch 'main' into codex/file-issue-agentos-3-for-ui-shell
6702ddb9 Merge pull request #118 from FlexNetOS/codex/update-agentos-4-documentation-tasks-list
30125d31 Add internal issue stubs for remaining gap remediation tasks
ee97af5a Link AGENTOS-4 to specific gap remediation tasks
6cc72730 Merge pull request #117 from FlexNetOS/codex/create-issue-agentos-2-for-agentos-server-profile
52580505 docs: add suggested task links
6afc23ca Merge pull request #116 from FlexNetOS/codex/create-gh-issue-for-agentos-1-task-5mdy5d
55882305 Deduplicate gap remediation task anchors
4fdcf1dc Deduplicate gap remediation task anchors
5c1bb827 Merge branch 'main' into codex/update-automation-master-plan-documentation
daf8013c Add automation master plan HITL coverage and tasks
160a72c9 docs: add suggested task links for automation plan
bd10570f Merge branch 'main' into codex/add-ai-assisted-task-prompts-to-project-management-app
f7e64095 feat: add ai assist prompt flow and roadmap exports
15691e1c Merge branch 'main' into codex/add-gap-remediation-tasks-section-in-markdown
474103c9 chore: script GitHub issue creation for agentos tasks
b51bfdd4 Merge pull request #94 from FlexNetOS/codex/edit-agentos-task-details-in-documentation
ce432a52 Update docs/plans/gap_remediation_tasks.md
82f9e146 Update docs/plans/gap_remediation_tasks.md
b7c52a99 Merge branch 'main' into codex/create-github-issues-for-agentos-tasks
b1b99be8 Track AGENTOS remediation tasks in issues
3291d7f8 Merge branch 'main' into codex/edit-agentos-task-details-in-documentation
f388e515 Add AGENTOS task overview anchors
9501cb97 Merge branch 'main' into codex/add-gap-remediation-tasks-section-in-markdown
6a0b9ad5 Add GitHub CLI installation helper
e550a117 Merge pull request #89 from FlexNetOS/codex/file-issue-for-agentos-6-features
cdbed298 Update docs/issues/AGENTOS-6.md
c2c6edc7 Update docs/issues/AGENTOS-6.md
69e251dc Update docs/issues/AGENTOS-6.md
81ed6157 Merge pull request #92 from FlexNetOS/codex/file-issue-agentos-3-for-ui-shell
02e1836c Merge pull request #91 from FlexNetOS/codex/create-issue-agentos-4-for-adaptive-orchestration
b0cf9d1c Update docs/issues/AGENTOS-4.md
2be30524 Merge pull request #90 from FlexNetOS/codex/create-issue-for-advanced-gateway-development
8d11ff73 Document AGENTOS-3 unified UI shell issue
3b31f97d Register AGENTOS-4 adaptive runtime orchestration issue
16600429 Add AGENTOS-5 advanced gateway issue summary
4d7b501b Add AGENTOS-6 issue for ecosystem launch
cb9ec6bb Merge pull request #88 from FlexNetOS/codex/add-gap-remediation-tasks-section-in-markdown
e6420d10 Fix AGENTOS-1 task title wording
3b4524cf Merge pull request #86 from FlexNetOS/codex/fix-firewall-rules-blocking-connections
a106ed67 Add script to disable Ubuntu ESM apt sources
c78b250c Restructure gap remediation tasks with objectives
44e10759 Add remediation task list for outstanding roadmap gaps
cf562579 Merge branch 'main' into codex/instrument-relocation-and-doc-pipelines
b4b504b1 Stabilize policy signatures and ledger handling
81d43a15 Merge branch 'main' into codex/model-service-dependencies-as-kernel-managed-capabilities
55882f48 Add kernel capability registry and portable tooling
5201614c Merge branch 'main' into codex/extend-configuration-with-automation-playbooks
34c38838 Refine automation telemetry configs and reporting
825232ba Update server/gateway/src/telemetry.rs
a2000cce Update server/gateway/src/auth.rs
b594b75b Merge branch 'main' into codex/expand-server-gateway-functionalities-eevpx3
14e648ce Tighten gateway auth, telemetry, and tests
4dbf0f0b Merge branch 'main' into codex/generate-documentation-infrastructure-and-workflows
1585a20e Revert "Add programmable gateway infrastructure"
07ea721c Merge branch 'main' into codex/expand-server-gateway-functionalities
53542ab3 Merge pull request #36 from FlexNetOS/codex/define-shared-design-system
be77d473 Merge branch 'main' into codex/implement-consolidated-schema-for-registry
6afe6cfe Add documentation sync agent and doc refresh pipeline
088fed0f Build registry knowledge graph service and tooling
52f04e26 Merge branch 'main' into codex/introduce-hardware-capability-detection
dea357e6 Merge pull request #35 from FlexNetOS/codex/implement-single-host-profile-configuration
a4ae6b24 Add programmable gateway orchestration
7f18eb66 Add hardware detection and runtime backend selection
5391c222 feat: add shared ui design system and streaming schema
219b837f feat: add single-host deployment profile
402dd310 Update docs/community/marketplace-contributions.md
dbe15cb8 Add blueprint, marketplace, analytics, and community docs
e2718cff Merge pull request #23 from FlexNetOS/codex/complete-phase-6-of-ui/ux-roadmap-a3kxl0
57c6f057 Update core/src/gateway.rs
23130def Update core/src/gateway.rs
368a64e2 Update core/src/gateway.rs
1c496b29 Update core/src/gateway.rs
5d06dd4c Clean gateway orchestration definitions
9d5b809a Finalize gateway automation fabric validation
414a102c Merge branch 'main' into codex/complete-phases-0-3-health-check-and-upgrade-code
04e32a20 Unify shell modules and expand unified API surface
3e1ce9fd Merge branch 'main' into codex/enhance-design-for-universal-gateway-u8kiu4
d6fce8fd Implement zero-trust routing intelligence for the gateway
b40b8288 Merge branch 'main' into codex/add-gateway-upgrades-and-features
f9375634 Enhance gateway with adaptive verification and governance
d594096d Merge pull request #13 from FlexNetOS/codex/add-upgrades-to-gateway-architecture
a12385d7 feat: upgrade gateway architecture
5ae273a1 Merge branch 'main' into codex/create-roadmap-for-integrated-ui-features-pcoj0l
1a2d04ae Implement persona-aware adapters and knowledge overlay
1e37c468 Merge branch 'main' into codex/create-roadmap-for-integrated-ui-features-gku3wk
b00b12ea Revamp unified control plane UI and API
dc20388b Merge branch 'main' into codex/enhance-design-for-universal-gateway-06p375
25030b2f Restore gitattributes and add WSL cargo activator
eb269432 Add gateway roadmap plan
ed4e5b69 Refine gateway orchestration and safety

## Recent Gateway Changes (2025-11-15)

1edfa93a fix: update documentation and scripts to use corepack for pnpm commands
ed4e63fd Merge branch 'main' into codex/extend-evidence-show-command-with-filters
69e53284 feat(cli): extend evidence ledger inspection
50d02b33 Merge branch 'main' into codex/populate-telemetry-with-sample-metrics
dfb3acd3 Merge branch 'main' into codex/extend-cicd-service-to-log-pipeline-events
397e353c feat: add gateway telemetry fixtures and budget summaries
ccb26327 feat(cicd): emit pipeline state ledger events
38ce37e1 Merge branch 'main' into codex/add-storage-doctor-subcommand-with-tests
f6411e13 Merge branch 'main' into codex/create-folders-and-seed-artifacts-for-analytics
2a8d74f1 feat(analytics): materialize goal metrics and expose context rewards
2daa7e82 Add storage doctor CLI and CI guard
f1233358 Merge branch 'main' into codex/update-storage-readme-and-add-stubs
bc24372a Merge branch 'main' into codex/add-notebook-specific-metrics-and-dashboards
76b1d2d5 Merge branch 'main' into codex/add-research-notebook-package-and-publish-bundle
868252ca docs: document instrumentation mirrors
8d01add0 feat(storage): add notebook analytics telemetry
379ac3f4 feat: add research notebook workspace suite
65c037b3 Merge branch 'main' into copilot/merge-unify-forked-repos
b9fadb58 Merge branch 'main' into codex/add-cli-targets-for-cargo-and-pnpm
784219fd feat: add memory module to workspace members
d05dfb29 Merge branch 'main' into codex/enforce-pnpm-usage-and-update-ide-guidance
5db5de05 Merge branch 'main' into codex/create-machine-readable-policy-file
31934e58 Update docs/automation/command-policy-migration.md
f1c44e17 Merge branch 'main' into codex/create-reusable-dev-environment-provisioning-scripts
43ca4797 Update docs/guides/dev-environment-cli.md
d9972823 Merge branch 'main' into codex/implement-event-driven-triage-service
216744b3 Update docs/automation/triage_service.md
be34bcf2 feat: enhance agent dispatch and reward systems with improved metadata resolution and approval requirements
f104a9f8 feat: add WSL interoperability runbook and GitHub CLI authentication helper script
0b5b1cf8 Merge branch 'main' into codex/enhance-ci/cd-processes-and-documentation
f92343e6 Update docs/autonomous_release_flow.md
64ef0051 Merge branch 'main' into codex/implement-capability-token-service-and-tests
2b1f0fca Merge branch 'main' into codex/extend-scorekeeper-for-weighted-rewards
0f4f235e Merge branch 'main' into codex/implement-self-status-endpoint-with-enhancements
5aa11d80 Merge branch 'main' into codex/revise-documentation-for-new-cli-tooling
30c81400 fix: normalize vscode archive bundle encoding
55123871 feat(devshell): add portable make targets
6f43740c Merge branch 'main' into codex/implement-self-status-endpoint-with-enhancements
e515e922 Update docs/self_status_reflection.md
d36e868b Merge branch 'main' into codex/update-documentation-and-architecture-diagrams
1c486ebf Merge branch 'main' into codex/create-tools.registry.json-and-extend-noa-cli
bfb9e3ea Merge branch 'main' into codex/add-north_star.deflex.json-and-schema-validation
0a6dd190 Update docs/trust/trust_pipeline.md
8449e440 Update core/src/gateway.rs
4bd41773 Merge branch 'main' into codex/implement-real-http-integrations-and-observability
d40c1091 Merge branch 'main' into codex/add-profiles-and-extend-kernel-profile-loading
c854c488 feat: implement local-first pipeline evidence validation and GitHub workflow integration
44da70bc feat: add hermetic execution targets document outlining CLI-first and offline-first objectives
791608e6 feat: add devshell pnpm enforcement
79f336dc feat: centralize command safety policy
1ff3f4ee feat: add manifest-driven cli dev environment tooling
769980d8 fix: replace triage archive binary with text snapshot
da300d8a feat: harden autonomous release with trust ledger
ef25222d feat(core): add capability token service and host control harness
af7134fc feat(workflow): finalize reward instrumentation
7bbcabcb feat: expose self status reflection and automated repairs
e24c44e0 docs: add agentic operator guide and lint checks
dca6a771 feat(cli): add registry-driven tooling surfaces
00af3946 fix(workflow): harden trust guardrails
9bc3ae02 Implement streaming providers with telemetry and failover
73b92772 Merge branch 'main' into codex/implement-sbom-and-audit-pipeline
87dbe56c Merge branch 'main' into codex/implement-world-graph-and-reconciler-logic
e46d863b Merge branch 'main' into codex/implement-quarantine-archiving-and-ci-checks
4f8e8f36 feat: add quarantine governance tooling
05e2b26d feat(core): issue profile-scoped capability tokens
8b11c7c6 feat: add reproducible audit pipeline tooling
b6f08a53 feat(kernel): add world graph reconciler and tests
96dc3ecf Merge branch 'main' into codex/enhance-code-for-production-readiness
9f1ad4c8 Update docs/tasks/ROADMAP_AGENTIC_KERNEL_TASK_LINKS.md
02ca54e6 Merge branch 'main' into codex/implement-agent-managed-pipelines-and-automation
9e983dd6 Expand Agentic Kernel task launch catalog
27a5a998 Merge pull request #177 from FlexNetOS/codex/update-inventory-role-references-and-sops
a33e641e Merge pull request #178 from FlexNetOS/codex/implement-automated-training-orchestration-in-ai
73d0346b fix: finalize agent managed workflow pipelines
27cc82e9 Merge remote-tracking branch 'origin/main'
d12f2c20 feat(ai): add training orchestrator and ml ops controls
7ae40a28 docs: align operational guides with agent hierarchy
f7f9db3b docs: refresh strategic roadmaps
889f6781 Merge branch 'main' into codex/add-new-capability-tokens-and-analytics-features
b8231d62 Merge branch 'main' into codex/expand-ai-database-with-new-tables-and-apis
6fd68c33 Merge branch 'feat/ci-guardrails-taskspec' into main
b705a8b1 feat: wire legacy feature flag into Cargo.toml and add LEGACY_INVENTORY.md scaffold
cb7454cc feat: add kanban goal insights analytics and autonomy
e8d5628d feat: persist workspace goal memory and planner insights
b4f2fed6 feat(ui): migrate vibe kanban to goal model
892c2005 feat(ui-api): add /api/uploads alias and /api/capabilities endpoint feat(orchestrator): support glob artifacts and maintain docs/tasks/index.json chore: reset docs/tasks/index.json placeholder
f7c8ba3c Merge branch 'main' into feat/ci-guardrails-taskspec
0e567bb7 feat(ui-api): wire Upload→Digest endpoint tests; keep receipts under crc/drop-in/incoming/receipts chore(workflow): TaskSpec acceptance uses noa_crc + noa_ui_api tests and ingest report ci: add Smoke Suite workflow to run CAS, Digest, Upload→Digest
c4291918 feat: stabilize instrumentation and wasm integrations
d886151b feat(ci): guardrails (deletion-lock, duplicate-fence, report-fence, change-budget) chore(ci): policy conformance (AGENT.md pointers) + single TS router enforcement feat(workflow): TaskSpec schema + validator; seed example and evidence ledger chore(staging): prior_projects lane skeleton
bf5c84d7 Merge branch 'main' into codex/archive-bespoke-router/controller-copies
4e9e9e66 Replace archived router snapshots with text copies
6dbd3c5e Merge pull request #155 from FlexNetOS/codex/create-documented-tasks-index-and-validation
899b86eb feat: add roadmap task index and validator
97d6ed28 feat(ui): add capability registry API and gating
d2cc0cdb Handle Ubuntu ESM .sources files in firewall fix script
bfdc7dbb Merge pull request #115 from FlexNetOS/codex/create-gh-issue-for-agentos-1-task
f4e706a7 Update docs/plans/gap_remediation_tasks.md
ba9041f0 Merge branch 'main' into codex/create-issue-agentos-4-for-adaptive-orchestration
4a74f788 Merge branch 'main' into codex/create-issue-for-advanced-gateway-development
e18fdd0e Enrich AGENTOS-5 suggested task links
7e891551 Link AGENTOS-4 issue to suggested tasks
0e086bab Merge branch 'main' into codex/file-issue-agentos-3-for-ui-shell
6702ddb9 Merge pull request #118 from FlexNetOS/codex/update-agentos-4-documentation-tasks-list
30125d31 Add internal issue stubs for remaining gap remediation tasks
ee97af5a Link AGENTOS-4 to specific gap remediation tasks
6cc72730 Merge pull request #117 from FlexNetOS/codex/create-issue-agentos-2-for-agentos-server-profile
52580505 docs: add suggested task links
6afc23ca Merge pull request #116 from FlexNetOS/codex/create-gh-issue-for-agentos-1-task-5mdy5d
55882305 Deduplicate gap remediation task anchors
4fdcf1dc Deduplicate gap remediation task anchors
5c1bb827 Merge branch 'main' into codex/update-automation-master-plan-documentation
daf8013c Add automation master plan HITL coverage and tasks
160a72c9 docs: add suggested task links for automation plan
bd10570f Merge branch 'main' into codex/add-ai-assisted-task-prompts-to-project-management-app
f7e64095 feat: add ai assist prompt flow and roadmap exports
15691e1c Merge branch 'main' into codex/add-gap-remediation-tasks-section-in-markdown
474103c9 chore: script GitHub issue creation for agentos tasks
b51bfdd4 Merge pull request #94 from FlexNetOS/codex/edit-agentos-task-details-in-documentation
ce432a52 Update docs/plans/gap_remediation_tasks.md
82f9e146 Update docs/plans/gap_remediation_tasks.md
b7c52a99 Merge branch 'main' into codex/create-github-issues-for-agentos-tasks
b1b99be8 Track AGENTOS remediation tasks in issues
3291d7f8 Merge branch 'main' into codex/edit-agentos-task-details-in-documentation
f388e515 Add AGENTOS task overview anchors
9501cb97 Merge branch 'main' into codex/add-gap-remediation-tasks-section-in-markdown
6a0b9ad5 Add GitHub CLI installation helper
e550a117 Merge pull request #89 from FlexNetOS/codex/file-issue-for-agentos-6-features
cdbed298 Update docs/issues/AGENTOS-6.md
c2c6edc7 Update docs/issues/AGENTOS-6.md
69e251dc Update docs/issues/AGENTOS-6.md
81ed6157 Merge pull request #92 from FlexNetOS/codex/file-issue-agentos-3-for-ui-shell
02e1836c Merge pull request #91 from FlexNetOS/codex/create-issue-agentos-4-for-adaptive-orchestration
b0cf9d1c Update docs/issues/AGENTOS-4.md
2be30524 Merge pull request #90 from FlexNetOS/codex/create-issue-for-advanced-gateway-development
8d11ff73 Document AGENTOS-3 unified UI shell issue
3b31f97d Register AGENTOS-4 adaptive runtime orchestration issue
16600429 Add AGENTOS-5 advanced gateway issue summary
4d7b501b Add AGENTOS-6 issue for ecosystem launch
cb9ec6bb Merge pull request #88 from FlexNetOS/codex/add-gap-remediation-tasks-section-in-markdown
e6420d10 Fix AGENTOS-1 task title wording
3b4524cf Merge pull request #86 from FlexNetOS/codex/fix-firewall-rules-blocking-connections
a106ed67 Add script to disable Ubuntu ESM apt sources
c78b250c Restructure gap remediation tasks with objectives
44e10759 Add remediation task list for outstanding roadmap gaps
cf562579 Merge branch 'main' into codex/instrument-relocation-and-doc-pipelines
b4b504b1 Stabilize policy signatures and ledger handling
81d43a15 Merge branch 'main' into codex/model-service-dependencies-as-kernel-managed-capabilities
55882f48 Add kernel capability registry and portable tooling
5201614c Merge branch 'main' into codex/extend-configuration-with-automation-playbooks
34c38838 Refine automation telemetry configs and reporting
825232ba Update server/gateway/src/telemetry.rs
a2000cce Update server/gateway/src/auth.rs
b594b75b Merge branch 'main' into codex/expand-server-gateway-functionalities-eevpx3
14e648ce Tighten gateway auth, telemetry, and tests
4dbf0f0b Merge branch 'main' into codex/generate-documentation-infrastructure-and-workflows
1585a20e Revert "Add programmable gateway infrastructure"
07ea721c Merge branch 'main' into codex/expand-server-gateway-functionalities
53542ab3 Merge pull request #36 from FlexNetOS/codex/define-shared-design-system
be77d473 Merge branch 'main' into codex/implement-consolidated-schema-for-registry
6afe6cfe Add documentation sync agent and doc refresh pipeline
088fed0f Build registry knowledge graph service and tooling
52f04e26 Merge branch 'main' into codex/introduce-hardware-capability-detection
dea357e6 Merge pull request #35 from FlexNetOS/codex/implement-single-host-profile-configuration
a4ae6b24 Add programmable gateway orchestration
7f18eb66 Add hardware detection and runtime backend selection
5391c222 feat: add shared ui design system and streaming schema
219b837f feat: add single-host deployment profile
402dd310 Update docs/community/marketplace-contributions.md
dbe15cb8 Add blueprint, marketplace, analytics, and community docs
e2718cff Merge pull request #23 from FlexNetOS/codex/complete-phase-6-of-ui/ux-roadmap-a3kxl0
57c6f057 Update core/src/gateway.rs
23130def Update core/src/gateway.rs
368a64e2 Update core/src/gateway.rs
1c496b29 Update core/src/gateway.rs
5d06dd4c Clean gateway orchestration definitions
9d5b809a Finalize gateway automation fabric validation
414a102c Merge branch 'main' into codex/complete-phases-0-3-health-check-and-upgrade-code
04e32a20 Unify shell modules and expand unified API surface
3e1ce9fd Merge branch 'main' into codex/enhance-design-for-universal-gateway-u8kiu4
d6fce8fd Implement zero-trust routing intelligence for the gateway
b40b8288 Merge branch 'main' into codex/add-gateway-upgrades-and-features
f9375634 Enhance gateway with adaptive verification and governance
d594096d Merge pull request #13 from FlexNetOS/codex/add-upgrades-to-gateway-architecture
a12385d7 feat: upgrade gateway architecture
5ae273a1 Merge branch 'main' into codex/create-roadmap-for-integrated-ui-features-pcoj0l
1a2d04ae Implement persona-aware adapters and knowledge overlay
1e37c468 Merge branch 'main' into codex/create-roadmap-for-integrated-ui-features-gku3wk
b00b12ea Revamp unified control plane UI and API
dc20388b Merge branch 'main' into codex/enhance-design-for-universal-gateway-06p375
25030b2f Restore gitattributes and add WSL cargo activator
eb269432 Add gateway roadmap plan
ed4e5b69 Refine gateway orchestration and safety

