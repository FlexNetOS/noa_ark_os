# Research Notebook Onboarding

The research notebook suite ships through the kernel marketplace and can be
activated per workspace using capability toggles. This guide walks operators
through enabling the bundle and verifying that the notebook module hydrates
across every surface.

## 1. Install the Marketplace Bundle

1. Open the marketplace catalog and locate **Research Notebook Experience Suite**.
2. Review the manifest at `apps/marketplace/catalog/research-notebook-suite.json`
   to confirm dependencies and entitlement scopes.
3. Launch the install workflow. The kernel applies the `register-ui-module`
   action which binds the notebook renderer to the web, desktop, and mobile
   adaptive stacks.
4. When prompted, allow the installer to fetch referenced media assets so the
   notebook gallery has preview art on first render.

## 2. Enable Workspace Toggles

Each workspace can opt in to the notebook capabilities after installation:

- **Enable research notebook surface** (`workspace.research.notebook`)
  - Grants access to the module in the navigation rail and workspace switcher.
  - Recommended roles: `researcher`, `admin`, and trusted operators assisting
    with notebook reviews.
- **Expose notebook media gallery** (`workspace.research.media-audit`)
  - Permits media streaming inside notebook detail views and ensures evidence
    attachments stay audit-ready.

Toggle both options from the workspace settings page or via the CLI:

```bash
kernel workspace toggle set --workspace research --toggle workspace.research.notebook --enabled true
kernel workspace toggle set --workspace research --toggle workspace.research.media-audit --enabled true
```

## 3. Verify Surfaces

- **Web**: confirm the `Research Notebook` module appears in the navigation rail
  and renders structured summaries, sections, citations, and media.
- **Desktop**: launch the Tauri shell; the notebook view should respect desktop
  gutters with the responsive registry defined in `ui/desktop/adaptive`.
- **Mobile**: open the React Native client; use the notebook tile to load the
  condensed stack layout emitted by `renderResearchNotebookMobile`.

## 4. Troubleshooting

- If the module does not appear, ensure the workspace has the required roles and
  that the marketplace install workflow completed without errors.
- To refresh renderer bindings after schema updates, rerun the marketplace
  install workflow or execute `kernel ui sync --module research-notebook`.
- Media placeholders indicate the CRC sync job has not completed; verify that
  `fetch-media-assets` succeeded and that the CRC endpoints are reachable from
  the client device.
