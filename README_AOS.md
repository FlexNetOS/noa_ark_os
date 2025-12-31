# AgenticOS Rollback & Watchdog Pack

This archive contains:
- `tools/aos-watchdog`: a simple health-check daemon that auto-rolls back on repeated failures.
- `tools/aos-ctl`: CLI to list slots, switch versions, and roll back.
- `tools/aos-installer`: installer that verifies bundle signatures and maintains `previous` symlink.
- `packaging/systemd/*.service`: systemd units to run the agent and watchdog.
- `mk/Makefile.aos`: Makefile snippet with targets to install and manage the units.

## Quick Start

```bash
# Build the tools
cargo build --release -p aos-ctl -p aos-watchdog -p aos-installer

# Install systemd units and binaries
sudo make -f mk/Makefile.aos install-units

# Check status
sudo aos-ctl status

# Roll back explicitly
sudo make -f mk/Makefile.aos rollback VERSION=2025.09.28

# Or roll back to 'previous'
sudo make -f mk/Makefile.aos rollback
```

## Notes
- Watchdog checks HTTP health endpoints first; if none provided, it runs `agent --version` from the current slot.
- Installer always sets `/opt/agenticos/previous` to the last active slot before switching.
- Everything is local-first and offline-friendly.
