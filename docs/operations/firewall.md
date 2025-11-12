# Firewall Troubleshooting: Ubuntu ESM APT Sources

GitHub-hosted runners and other restricted environments block outbound
connections to `esm.ubuntu.com`. When `apt update` or `apt install`
attempts to reach Ubuntu's Extended Security Maintenance (ESM) endpoints,
the command fails with a firewall warning similar to:

```
Warning: Firewall rules blocked me from connecting to one or more addresses
    esm.ubuntu.com
        Triggering command: /usr/lib/apt/methods/https (dns block)
```

To prevent the package manager from attempting those connections, disable
the ESM apt sources before running any `apt` commands.

## Quick Fix Script

Run the helper script from the repository root:

```bash
./scripts/fix/disable_ubuntu_esm.sh
```

The script:

- Requests elevated privileges (re-invoking itself with `sudo` when
  available).
- Backs up every modified file to `/etc/apt/noa-backups/` with a timestamp.
- Comments out any non-disabled entries that reference `esm.ubuntu.com` in
  `/etc/apt/sources.list` and `/etc/apt/sources.list.d/*.list`.
- Neutralizes the `20apt-esm-hook.conf` file so future `apt update` runs do
  not re-enable the hook.

After the script reports success, rerun your package command (for example
`sudo apt-get update`). The firewall warning should no longer appear because
`apt` will avoid the blocked ESM endpoints.

## Verification

You can confirm the change by running:

```bash
grep -R "esm.ubuntu.com" /etc/apt/sources.list /etc/apt/sources.list.d
```

Only commented entries prefixed with `# disabled-by-noa` should remain.

If you ever need to restore the previous configuration, copy the relevant
backup file from `/etc/apt/noa-backups/` back to its original location.
