# Truth Gate Checklist — Portable PowerShell Bundle

- [x] Artifacts hashed: manifest + setup scripts + launcher + llama helpers recorded with fresh SHA-256 hashes inside `evidence.ledger.json` (see A1–A10).
- [x] Smoke tests executed on 2025-11-21: `python3 tools/automation/check_portable_pwsh.py --platform linux-x64 --ensure-exec --require-current --output json`, `scripts/lib/ensure_no_duplicate_tasks.sh`, `$POWERSHELL_BIN -File scripts/dev/setup-llama-cpp.ps1 -SkipModelDownload`, and `bash scripts/full_stack_launch.sh --prepare-only --skip-tests --skip-docker --skip-make --skip-notebook --cuda-mode auto --llama-mode auto --master-controller-mode skip --pipeline-mode skip` all exited 0 (launcher run captured the new env-flag summary at 15:59Z).
- [x] Requirements ↔ artifacts mapped in `claims.table.md` with updated manifest hash + launcher evidence + Linux CUDA/Llama coverage.
- [x] Limits captured per-claim: desktop bundles not yet run on native hosts, placeholder GGUF is non-production, and inference still needs real checkpoints and GPU drivers.
- [x] Evidence ledger updated with command transcripts + timestamps + notes for checker, duplicate guard, llama setup, and launcher.
- [x] Gap scan: flagged that desktop archives are packaged but untested locally and that placeholder GGUF must be replaced before production inference.
- [x] Triple-Verification — Pass A: manifest + activator code review; Pass B: reran launcher/llama setup; Pass C: duplicate-task guard enforces manifest claim before pipeline work.
