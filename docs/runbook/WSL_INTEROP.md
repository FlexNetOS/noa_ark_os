# WSL Interoperability Runbook

Hermetic tooling assumes WSL can reach the Windows host when a workflow requests the portable PowerShell toolchain (for example, opening a browser during `gh auth login --web`). When interoperability is disabled, commands fail with errors such as:

```
grep: /proc/sys/fs/binfmt_misc/WSLInterop: No such file or directory
WSL Interoperability is disabled. Please enable it before using WSL.
```

This guide explains how to diagnose and restore interoperability while still keeping a Linux-only fallback.

## 1. Detecting the Current State

Run the helper script from the workspace root:

```bash
./scripts/tools/gh-auth-login.sh --check 2>/dev/null || true
```

Or manually inspect the kernel switch:

```bash
cat /proc/sys/fs/binfmt_misc/WSLInterop 2>/dev/null
```

- File present and containing `enabled`: interoperability is active.
- File missing or showing `disabled`: interoperability is off.

## 2. Re-Enabling Interop

1. **Update `/etc/wsl.conf` inside the distro** (requires sudo):

   ```bash
   sudo tee /etc/wsl.conf >/dev/null <<'CONF'
   [interop]
   enabled=true
   appendWindowsPath=true
   CONF
   ```

2. **Shut down and restart WSL from Windows** (PowerShell or Command Prompt as administrator):

   ```powershell
   wsl --shutdown
   wsl --update
   wsl --set-default-version 2
   ```

3. **Launch the distro again** (e.g., `wsl` or Windows Terminal). On start, verify the file exists:

   ```bash
   cat /proc/sys/fs/binfmt_misc/WSLInterop
   ```

4. If the file is still absent, ensure the Windows features *Virtual Machine Platform* and *Windows Subsystem for Linux* are enabled via `OptionalFeatures.exe`, then reboot Windows and repeat steps 2–3.

## 3. Fallback When Interop Is Unavailable

Some flows (like CI agent logins) can operate without Windows. Use the automation script instead of invoking `gh auth login` directly:

```bash
./scripts/tools/gh-auth-login.sh
```

The helper automatically:

- Detects whether `/proc/sys/fs/binfmt_misc/WSLInterop` is enabled.
- Uses the regular `--web` login when interoperability works.
- Falls back to the device-code login (`--device`) when interop is disabled, avoiding the browser call that triggers the error.

Environment overrides:

- `GH_LOGIN_MODE=device` forces device flow.
- `GH_LOGIN_MODE=web` forces the browser flow (will fail if interop is actually disabled).
- `GH_SCOPES`, `GH_GIT_PROTOCOL`, and `GH_HOST` can be tweaked for custom auth scopes.

## 4. Checklist Before Running Hermetic Pipelines

- [ ] `./scripts/tools/gh-auth-login.sh` completes without WSL warnings.
- [ ] `source ./server/tools/activate-cargo-wsl.sh` works with or without Windows path injection.
- [ ] `wslview` opens a browser (only required if you prefer the `--web` GitHub auth flow).
- [ ] `make pipeline.local` runs from within the Linux distro without hitting Windows binaries directly.

With these steps, the automation can continue “as first designed”: Linux-only commands remain hermetic, while Windows interoperability can be re-enabled whenever the workflow benefits from it.
