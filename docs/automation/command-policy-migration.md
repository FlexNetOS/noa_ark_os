# Command Policy Migration Guide

The workspace now centralises terminal safety rules in [`tools/automation/policy.yaml`](../../tools/automation/policy.yaml). The policy powers both IDE integrations and headless automation so contributors only maintain a single source of truth.

## Why the change?

* **Parity across environments:** headless tools such as `scripts/guardrails-local.sh` and the offline PR queue now consult the same policy used by VS Code auto-approval.
* **Simpler reviews:** policy updates appear as standalone YAML edits instead of sprawling JSON diffs in `.vscode/settings.json`.
* **Future automation:** other shells, editors, and agents can adopt the policy file without re-encoding patterns.

## Updating the policy

1. Edit `tools/automation/policy.yaml`. JSON syntax is valid YAML, so formatting can stay consistent with the existing rules.
2. Preserve rule ordering—deny rules should appear before their corresponding allow rules when both apply to the same command.
3. Run the validator to confirm your rule behaves as expected:

   ```bash
   # Example: Allowed command (matches "git status" rule)
   pnpm exec tsx tools/automation/validate-command.ts --explain "git status -sb"
   # Output: ✅ Allowed by rule: "git status"

   # Example: Blocked command (not covered by policy)
   pnpm exec tsx tools/automation/validate-command.ts --explain "git status --dangerous-flag"
   # Output: ❌ Blocked: no matching allow rule
4. Commit the policy change alongside any dependent script or documentation updates.

## Synchronising VS Code settings

The IDE block `chat.tools.terminal.autoApprove` mirrors the policy file for convenience. When changing the policy:

1. Update the YAML first.
2. Copy the relevant rule into `.vscode/settings.json` so the editor stays aligned until we automate sync.
3. Reference the migration commit or pull request in the settings file comment if a reviewer needs extra context.

## Integrations that now enforce the policy

* `tools/automation/validate-command.ts` – CLI helper used across automation.
* `scripts/guardrails-local.sh` – validates internal `git` commands before execution.
* `tools/offline_pr_queue` – validates all Git and auxiliary commands run during offline PR handling.

When wiring new tooling into the workspace, import the helper exported from `tools/automation/command_policy.ts` or shell out to `validate-command.ts` so every surface respects the central policy.
