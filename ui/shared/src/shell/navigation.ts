import { EntryPoint, SurfaceKind } from "./types";

export interface NavigationInventory {
  entryPoints: EntryPoint[];
  surfaces: SurfaceKind[];
}

export const navigationInventory: NavigationInventory = {
  surfaces: ["web", "desktop", "cli"],
  entryPoints: [
    {
      id: "dashboard",
      label: "Operations Dashboard",
      description: "Monitor kernel, runtime, and gateway health.",
      route: "/dashboard",
      surfaces: ["web", "desktop"],
      category: "core",
    },
    {
      id: "workflows",
      label: "Workflow Console",
      description: "Resume or observe orchestrated workflows across environments.",
      route: "/workflows",
      surfaces: ["web", "desktop", "cli"],
      category: "core",
    },
    {
      id: "gateway-observability",
      label: "Gateway Observability",
      description: "Inspect ingress metrics, policies, and recent incidents.",
      route: "/gateway",
      surfaces: ["web", "desktop"],
      category: "observability",
    },
    {
      id: "ecosystem-catalog",
      label: "Ecosystem Catalog",
      description: "Install partner integrations and workflow bundles.",
      route: "/ecosystem",
      surfaces: ["web", "desktop"],
      category: "ecosystem",
    },
    {
      id: "cli-session",
      label: "Shell Session",
      description: "Terminal-first interface for incident response and automation.",
      route: "agentos shell",
      surfaces: ["cli"],
      category: "core",
    },
  ],
};

export function entryPointsForSurface(surface: SurfaceKind): EntryPoint[] {
  return navigationInventory.entryPoints.filter((entry) => entry.surfaces.includes(surface));
}
