import { entryPointsForSurface, navigationInventory } from "./navigation";
import { ShellStateManager, ShellStateOptions } from "./state";
import { TelemetryClient } from "./telemetry";
import { AuthProvider, LocalAuthProvider, OidcAuthProvider } from "./auth";
import { ComponentBinding, EntryPoint, ShellConfiguration, SurfaceKind } from "./types";

export interface UnifiedShellOptions extends Omit<ShellStateOptions, "telemetry" | "authProviders"> {
  telemetry?: TelemetryClient;
  authProviders?: AuthProvider[];
  configuration?: Partial<ShellConfiguration>;
}

function defaultConfiguration(): ShellConfiguration {
  return {
    entryPoints: navigationInventory.entryPoints,
    componentBindings: [
      { widget: "header.brand", surfaces: ["web", "desktop"], implementation: "@noa-ark/shared-ui/components/Header" },
      { widget: "session.switcher", surfaces: ["web", "desktop", "cli"], implementation: "@noa-ark/shared-ui/components/SessionSwitcher" },
      { widget: "gateway.metrics", surfaces: ["web", "desktop"], implementation: "@noa-ark/shared-ui/components/GatewayMetrics" },
      { widget: "ecosystem.catalog", surfaces: ["web"], implementation: "@noa-ark/shared-ui/components/EcosystemCatalog" },
      { widget: "cli.prompt", surfaces: ["cli"], implementation: "@noa-ark/shared-ui/components/CliPrompt" },
    ],
    defaultSurface: "web",
  };
}

function mergeConfiguration(base: ShellConfiguration, override?: Partial<ShellConfiguration>): ShellConfiguration {
  if (!override) {
    return base;
  }

  return {
    entryPoints: override.entryPoints ?? base.entryPoints,
    componentBindings: override.componentBindings ?? base.componentBindings,
    defaultSurface: override.defaultSurface ?? base.defaultSurface,
  };
}

export class UnifiedShell {
  private readonly config: ShellConfiguration;
  private readonly stateManager: ShellStateManager;
  private readonly telemetry: TelemetryClient;
  private readonly surfaces = new Set<SurfaceKind>();

  constructor(private readonly options: UnifiedShellOptions) {
    const defaultConfig = defaultConfiguration();
    this.config = mergeConfiguration(defaultConfig, options.configuration);
    const telemetry = options.telemetry ?? new TelemetryClient();
    const providers = options.authProviders ?? [
      new LocalAuthProvider(),
      new OidcAuthProvider("oidc", { issuer: "https://id.local", clientId: "agentos" }),
    ];

    this.stateManager = new ShellStateManager({
      workflowEndpoint: options.workflowEndpoint,
      telemetry,
      authProviders: providers,
    });
    this.telemetry = telemetry;
    this.registerSurface(this.config.defaultSurface);
  }

  registerSurface(surface: SurfaceKind): void {
    if (!navigationInventory.surfaces.includes(surface)) {
      throw new Error(`Unsupported surface: ${surface}`);
    }
    this.surfaces.add(surface);
  }

  availableSurfaces(): SurfaceKind[] {
    return [...this.surfaces];
  }

  entryPoints(surface?: SurfaceKind): EntryPoint[] {
    if (!surface) {
      return this.config.entryPoints;
    }
    return entryPointsForSurface(surface);
  }

  componentFor(widget: string, surface: SurfaceKind): ComponentBinding | undefined {
    return this.config.componentBindings.find(
      (binding) => binding.widget === widget && binding.surfaces.includes(surface),
    );
  }

  async signIn(providerName: string): Promise<void> {
    await this.stateManager.signIn(providerName);
  }

  start(): void {
    this.stateManager.connect();
  }

  stop(): void {
    this.stateManager.disconnect();
  }

  navigate(surface: SurfaceKind, entryPointId: string): EntryPoint {
    if (!this.surfaces.has(surface)) {
      throw new Error(`Surface ${surface} not registered`);
    }
    const entry = this.config.entryPoints.find((item) => item.id === entryPointId);
    if (!entry || !entry.surfaces.includes(surface)) {
      throw new Error(`Entry point ${entryPointId} not available on ${surface}`);
    }
    this.stateManager.recordNavigation(surface, entry);
    return entry;
  }

  telemetryClient(): TelemetryClient {
    return this.telemetry;
  }

  componentBindings(): ComponentBinding[] {
    return this.config.componentBindings;
  }

  currentSessionProvider(): string | undefined {
    return this.stateManager.currentSession()?.provider;
  }
}
