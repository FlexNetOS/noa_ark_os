"use client";

import { AnalyticsPanel } from "./components/AnalyticsPanel";
import { AssistPanel } from "./components/AssistPanel";
import { BoardShell } from "./components/BoardShell";
import { NotificationCenter } from "./components/NotificationCenter";
import { PresenceBar } from "./components/PresenceBar";
import { ActivityTimeline } from "./components/ActivityTimeline";
import { IntegrationStatus } from "./components/IntegrationStatus";
import { WorkspaceSwitcher } from "./components/WorkspaceSwitcher";
import { useBoardState } from "./components/useBoardState";
import { useSession } from "./components/useSession";

export default function Page() {
  const session = useSession();
  const state = useBoardState(session.user);

  const ready = session.status === "ready" && !!session.user && state.hydrated && state.snapshot;

  if (session.status === "loading") {
    return <FullScreenMessage label="Initializing workspace…" />;
  }

  if (!session.user) {
    return <SignInScreen onSignIn={session.signIn} />;
  }

  if (!ready) {
    return <FullScreenMessage label="Syncing boards from the NOA ARK cloud…" />;
  }

  return (
    <div className="relative min-h-screen bg-background pb-32 text-white">
      <NotificationCenter notifications={state.notifications} onDismiss={state.dismissNotification} />

      <header className="border-b border-white/10 bg-surface/50">
        <div className="mx-auto flex max-w-7xl flex-col gap-4 px-6 py-6 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p className="text-xs uppercase tracking-[0.4em] text-white/40">NOA ARK OS</p>
            <h1 className="text-2xl font-semibold tracking-tight text-white">Vibe Kanban Control Hub</h1>
            <p className="text-sm text-white/50">
              Connected to workspaces, live presence, and agent intelligence.
            </p>
          </div>
          <div className="flex items-center gap-4">
            <div className="text-right">
              <p className="text-sm font-semibold text-white">{session.user.name}</p>
              <p className="text-xs uppercase tracking-[0.2em] text-white/40">Collaborator</p>
            </div>
            <div className="flex h-12 w-12 items-center justify-center rounded-full bg-gradient-to-br from-indigo-500 via-purple-500 to-blue-500 text-lg font-semibold">
              {session.user.name.slice(0, 2).toUpperCase()}
            </div>
          </div>
        </div>
      </header>

      <main className="relative mx-auto grid max-w-7xl gap-6 px-6 py-12 lg:grid-cols-[320px_1fr_320px]">
        <div className="space-y-6">
          <WorkspaceSwitcher state={state} />
          <IntegrationStatus integrations={state.integrations} />
        </div>
        <div className="space-y-6">
          <BoardShell state={state} />
        </div>
        <div className="space-y-6">
          <PresenceBar presence={state.presence} members={state.workspace?.members ?? []} />
          <AssistPanel assist={state.assist} onRequest={state.requestAssist} />
          <AnalyticsPanel board={state.snapshot} />
          <ActivityTimeline activity={state.activity} />
        </div>
      </main>
    </div>
  );
}

type FullScreenMessageProps = {
  label: string;
};

function FullScreenMessage({ label }: FullScreenMessageProps) {
  return (
    <div className="flex min-h-screen items-center justify-center bg-background">
      <div className="relative flex items-center gap-3 rounded-full border border-white/10 bg-surface/80 px-6 py-3 text-sm font-medium text-white/60">
        <span className="h-2.5 w-2.5 animate-pulse rounded-full bg-gradient-to-r from-indigo-500 via-purple-500 to-blue-500" />
        {label}
      </div>
    </div>
  );
}

type SignInScreenProps = {
  onSignIn: (name: string) => Promise<void>;
};

function SignInScreen({ onSignIn }: SignInScreenProps) {
  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const formData = new FormData(event.currentTarget);
    const name = String(formData.get("name") ?? "").trim();
    if (!name) return;
    await onSignIn(name);
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-gradient-to-br from-indigo-950 via-purple-950 to-black p-6 text-white">
      <form
        onSubmit={handleSubmit}
        className="w-full max-w-md space-y-6 rounded-3xl border border-white/10 bg-surface/70 p-10 text-white backdrop-blur-xl"
      >
        <div>
          <p className="text-xs uppercase tracking-[0.4em] text-white/40">Vibe workspace</p>
          <h1 className="mt-2 text-3xl font-semibold">Sign in to sync your board</h1>
          <p className="mt-2 text-sm text-white/60">
            Create a session to join the live workspace and collaborate with your crew.
          </p>
        </div>
        <div className="space-y-3">
          <label className="block text-xs uppercase tracking-[0.3em] text-white/40" htmlFor="name">
            Display name
          </label>
          <input
            id="name"
            name="name"
            placeholder="Ava, Kai, Sol…"
            className="w-full rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm text-white focus:outline-none"
          />
        </div>
        <button
          type="submit"
          className="w-full rounded-full border border-indigo-400/40 bg-indigo-500/20 px-6 py-3 text-sm font-semibold text-indigo-100 transition hover:border-indigo-300/60 hover:bg-indigo-500/30"
        >
          Enter workspace
        </button>
      </form>
    </div>
  );
}
