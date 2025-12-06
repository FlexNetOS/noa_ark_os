export type SurfaceKind = "web" | "desktop" | "cli";

export interface EntryPoint {
  id: string;
  label: string;
  description: string;
  route: string;
  surfaces: SurfaceKind[];
  category: "core" | "observability" | "ecosystem";
}

export interface ComponentBinding {
  widget: string;
  surfaces: SurfaceKind[];
  implementation: string;
}

export interface ShellConfiguration {
  entryPoints: EntryPoint[];
  componentBindings: ComponentBinding[];
  defaultSurface: SurfaceKind;
}

export interface ShellTelemetryEvent {
  surface: SurfaceKind;
  entryPointId: string;
  event: "navigate" | "error" | "action";
  timestamp: number;
  metadata?: Record<string, unknown>;
}

export interface AuthSession {
  token: string;
  user: {
    id: string;
    name: string;
    email: string;
  };
  provider: string;
  expiresAt: number;
}
