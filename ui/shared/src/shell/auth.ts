import { AuthSession } from "./types";

export interface AuthProvider {
  readonly name: string;
  readonly supportsOffline: boolean;
  signIn(): Promise<AuthSession>;
  refresh(session: AuthSession): Promise<AuthSession>;
  signOut(session: AuthSession): Promise<void>;
}

export class LocalAuthProvider implements AuthProvider {
  readonly name = "local";
  readonly supportsOffline = true;

  async signIn(): Promise<AuthSession> {
    const now = Date.now();
    return {
      token: "local-token",
      provider: this.name,
      user: { id: "local-user", name: "Local User", email: "local@example.com" },
      expiresAt: now + 8 * 60 * 60 * 1000,
    };
  }

  async refresh(session: AuthSession): Promise<AuthSession> {
    return { ...session, expiresAt: Date.now() + 8 * 60 * 60 * 1000 };
  }

  async signOut(_session: AuthSession): Promise<void> {
    return;
  }
}

export interface OidcProviderOptions {
  issuer: string;
  clientId: string;
}

export class OidcAuthProvider implements AuthProvider {
  readonly supportsOffline = false;
  constructor(readonly name: string, private readonly options: OidcProviderOptions) {}

  async signIn(): Promise<AuthSession> {
    const now = Date.now();
    return {
      token: `${this.name}-token`,
      provider: this.name,
      user: { id: `${this.name}-user`, name: "OIDC User", email: `${this.name}@example.com` },
      expiresAt: now + 60 * 60 * 1000,
    };
  }

  async refresh(session: AuthSession): Promise<AuthSession> {
    return { ...session, expiresAt: Date.now() + 60 * 60 * 1000 };
  }

  async signOut(_session: AuthSession): Promise<void> {
    return;
  }
}

export class AuthRegistry {
  private readonly providers = new Map<string, AuthProvider>();

  register(provider: AuthProvider): void {
    this.providers.set(provider.name, provider);
  }

  get(name: string): AuthProvider {
    const provider = this.providers.get(name);
    if (!provider) {
      throw new Error(`Unknown auth provider: ${name}`);
    }
    return provider;
  }

  list(): AuthProvider[] {
    return [...this.providers.values()];
  }
}
