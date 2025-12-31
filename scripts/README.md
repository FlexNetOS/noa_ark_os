## Scripts

Automation scripts for NOA ARK OS.

## Structure
- build/ - Build automation
- dev/ - Development helpers
- integration/ - Integration scripts
- maintenance/ - Cleanup and backup
- fix/ - Environment repair utilities (e.g. disable Ubuntu ESM apt sources)
- tools/ - Tool configuration
- testing/ - Test runners

## Usage
Always run from workspace root.
Activate portable Cargo first for Rust scripts.

## Featured Entry Points

- `scripts/full_stack_launch.sh` &mdash; all-in-one launcher that runs dependency prep, kernel hardening, Docker bootstrapping, llama/CUDA provisioning, optional master controller automation, and pipeline evidence capture. Use the new `--*-mode {auto|force|skip}` flags to control each phase and rely on the summary output to confirm what ran.

## UI / Tooling Inventory

See [`scripts/INTEGRATION_STATUS.md`](./INTEGRATION_STATUS.md) for the authoritative table that maps every script in this directory to its frontend tool exposure status.
