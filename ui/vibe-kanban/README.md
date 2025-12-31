# Vibe Kanban Workspace

## Manual verification: CRC upload notifications

1. Launch the UI (`pnpm --filter vibe-kanban dev`) alongside the UI API server so the `/ui/drop-in/upload` bridge is available.
2. Sign in to the workspace and locate the **CRC Uploads** panel in the right rail.
3. Choose a supported drop type, attach a file (e.g., a `.zip` or `.tar.gz` archive), and click **Send to CRC**.
4. Confirm the toast bubble lists the returned CAS hash(es) and links to the generated receipt file.
5. Inspect the workspace data file (`ui/vibe-kanban/data/workspaces.json`) to ensure the new `uploadReceipts` entry and persisted notification are recorded for offline replay.
