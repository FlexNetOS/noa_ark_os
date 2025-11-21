# Truth Gate Checklist — Portable PowerShell Bundle

- [x] Artifacts hashed: manifest + activation/evidence hashes captured in `evidence.ledger.json`.
- [x] Smoke tests executed: `tools/automation/check_portable_pwsh.py --output text`, `scripts/lib/ensure_no_duplicate_tasks.sh`, and `scripts/full_stack_launch.sh --prepare-only --skip-tests --skip-docker --skip-make --skip-notebook --cuda-mode auto --llama-mode auto --master-controller-mode skip --pipeline-mode skip` all exited 0 on 2025-11-21.
- [x] Requirements ↔ artifacts mapped in `claims.table.md`.
- [x] Limits and unsupported paths documented per-claim (Windows/macOS bundle gaps, CUDA/llama Windows-only provisioning).
- [x] Evidence ledger includes file hashes + command metadata for this task.
- [x] Gap scan noted the remaining Windows/macOS provisioning TODO plus CUDA/llama platform constraint.
- [x] Triple-Verification: Pass A (self-check via manifest + activator review), Pass B (reran launcher to confirm activation evidence), Pass C (adversarial guard by forcing duplicate-task checker to verify manifest integrity).
