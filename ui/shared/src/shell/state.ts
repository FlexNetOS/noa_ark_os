import { SessionContinuityClient } from "../session";
import { TelemetryClient } from "./telemetry";
import { AuthRegistry, AuthProvider } from "./auth";
import { AuthSession, EntryPoint, ShellTelemetryEvent, SurfaceKind } from "./types";

export interface ShellStateOptions {
  workflowEndpoint: string;
  authProviders?: AuthProvider[];
  telemetry?: TelemetryClient;
}

export class ShellStateManager {
  private readonly continuity: SessionContinuityClient;
  private readonly telemetry: TelemetryClient;
  private readonly authRegistry = new AuthRegistry();
  private session?: AuthSession;

  constructor(private readonly options: ShellStateOptions) {
    this.continuity = new SessionContinuityClient({ workflowEndpoint: options.workflowEndpoint });
    this.telemetry = options.telemetry ?? new TelemetryClient();
    for (const provider of options.authProviders ?? []) {
      this.authRegistry.register(provider);
    }
  }

  connect(): void {
    this.continuity.connectWebSocket();
  }

  disconnect(): void {
    this.continuity.disconnect();
  }

  async signIn(providerName: string): Promise<AuthSession> {
    const provider = this.authRegistry.get(providerName);
    this.session = await provider.signIn();
    return this.session;
  }

  currentSession(): AuthSession | undefined {
    return this.session;
  }

  recordNavigation(surface: SurfaceKind, entryPoint: EntryPoint): void {
    const event: ShellTelemetryEvent = {
      surface,
      entryPointId: entryPoint.id,
      event: "navigate",
      timestamp: Date.now(),
      metadata: { route: entryPoint.route },
    };
    this.telemetry.record(event);
  }

  telemetryClient(): TelemetryClient {
    return this.telemetry;
  }
}
