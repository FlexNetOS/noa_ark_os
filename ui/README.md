# Dynamic UI/UX System

Multi-platform, adaptive user interface system that exposes one unified shell
API across Rust and TypeScript, then fans out to platform-specific adapters.

## Implemented Surfaces (Current State)

- **Rust UI Core (`ui/core`)**
  - `noa_ui` crate with:
    - `Platform` enum (Server, Mobile, Desktop, Web, ARGlasses, XRHeadset)
    - `Capability` flags (Touch, Mouse, Keyboard, Voice, Gesture, SpatialTracking, HandTracking, EyeTracking)
    - `UIContext` and `UIState` for platform-aware rendering
    - `UnifiedShell` orchestrator wiring navigation, chat, analytics, knowledge base, and modules
  - Adapters in `ui/core/src/adapters.rs`:
    - `ServerAdapter` – server-only orchestration surfaces
    - `ReactAdapter` – desktop/web React shell via `ui/noa-dashboard`
    - `TauriAdapter` – desktop shell, reading `apps/desktop-shell/tauri.conf.json`
    - `ReactNativeAdapter` – mobile shell, reading `apps/mobile-shell/app.json`
    - `SpatialAdapter` – XR shell, reading `apps/xr-shell/scene.graph.json`

- **Shared UI (`ui/shared`)**
  - Design tokens and schemas (`tokens.ts`, `schema.ts`)
  - Shell and navigation (`shell/unifiedShell.ts`, `shell/state.ts`, `shell/telemetry.ts`)
  - Renderers:
    - `renderers/web.tsx` – web React renderer
    - `renderers/native.ts` – React Native tree builder
    - `renderers/xr.ts` – projects page schemas into spatial scenes for XR engines

- **Web Workspace (`ui/vibe-kanban`)**
  - Next.js 14 app for the Vibe Kanban board, integrated with:
    - `server/ui_api` for shell state and capabilities
    - Shared UI components from `@noa-ark/shared-ui`
    - CRC upload bridge (`CRC Uploads` panel) to the artifact pipeline
  - Verified by `npx vitest run` (all tests passing).

- **Adaptive Research Notebook Renderers**
  - `ui/web/adaptive/research-notebook.tsx` – web renderer binding shared UI and the research notebook schema
  - `ui/desktop/adaptive/research-notebook.tsx` – desktop-specific layout using the shared web renderer
  - `ui/mobile/adaptive/research-notebook.ts` – native tree builder for mobile shells

## Architecture (Actual Layout)

```text
ui/
├── core/              # Rust unified shell (noa_ui crate)
├── shared/            # Shared TS design system + shell
├── vibe-kanban/       # Next.js web workspace
├── research-notebook/ # Web research notebook shell
├── desktop/           # Desktop adaptive renderers (TS, research notebook)
├── mobile/            # Mobile adaptive renderers (TS, research notebook)
├── web/               # Web adaptive renderers (TS, research notebook)
└── NOA API / dashboards
    ├── noa-api/       # API docs/shell assets
    ├── noa-dashboard/ # Dashboard assets
    └── dashboard/     # Legacy/experimental dashboard
```

Upstream application shells live under `apps/`:

```text
apps/
├── desktop-shell/     # Tauri desktop host
├── mobile-shell/      # React Native mobile host
└── xr-shell/          # XR host consuming spatial scenes
```

The Rust adapters in `ui/core` mount against these manifests, so hardware-
specific UI stacks still pass through the same unified shell and state model.

## Cross-Platform & Hardware Integration

- Platform-specific initialization uses `Platform` + `Capability` to shape
  `UIContext` (screen size, DPI, input and tracking capabilities).
- `UnifiedShell::render` takes a `PlatformAdapter` trait object, so the same
  module graph and state can be rendered to:
  - Server-only surfaces (`ServerAdapter`)
  - Browser/Tauri windows (`ReactAdapter`, `TauriAdapter`)
  - React Native views (`ReactNativeAdapter`)
  - XR spatial graphs (`SpatialAdapter` + `ui/shared/src/renderers/xr.ts`)
- `ui/shared` surfaces (`SurfaceKind` in TS) mirror these targets on the web/
  native side, so TypeScript hosts can share entrypoints and component bindings
  with the Rust shell.

This gives one conceptual shell with multiple hardware-backed skins, rather
than separate, divergent UI stacks per platform.

## Gaps and Roadmap (Compared to Earlier Docs)

To stay aligned with AGENT.md (no silent decay), the following are known gaps:

- The older architecture sketch referred to `ui/server`, `ui/ar`, `ui/xr`,
  `ui/components`, and `ui/renderer` directories. The actual layout uses
  `ui/core`, `ui/shared`, and platform adapters, with AR/XR integration
  handled via `ui/shared/src/renderers/xr.ts` and `apps/xr-shell/`.
- Mobile and desktop adaptive renderers currently cover the research notebook
  surfaces; full shell parity (including kanban and other modules) is still
  future work.
- Server interface responsibilities (REST/GraphQL/WebSocket/gRPC) are owned by
  `server/ui_api` and gateway layers, not by a `ui/server/` tree.

Future cross-platform work should:

- Extend the shared `UnifiedShell` and `ui/shared` renderers to cover more
  modules (e.g., kanban, dashboards) across mobile/desktop/web/XR.
- Keep this README in sync with the actual directory structure and adapter
  code, treating it as a reflection of `ui/core`, `ui/shared`, and `apps/*`
  rather than an aspirational sketch.
