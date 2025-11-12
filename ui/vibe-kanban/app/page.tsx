"use client";

import { useEffect, useMemo, useState } from "react";
import type { PageEnvelope, ResumeToken } from "@noa-ark/shared-ui/schema";
import { createSchemaClient, SessionContinuityClient } from "@noa-ark/shared-ui/session";
import { vibeDashboardEnvelope } from "@noa-ark/shared-ui/samples";

import { NotificationCenter } from "./components/NotificationCenter";
import { SchemaDrivenRenderer } from "./components/SchemaDrivenRenderer";
import { useBoardState } from "./components/useBoardState";
import { useSession } from "./components/useSession";

export default function Page() {
  const session = useSession();
  const state = useBoardState(session.user);
  const [envelope, setEnvelope] = useState<PageEnvelope>(vibeDashboardEnvelope);
  const [resumeToken, setResumeToken] = useState<ResumeToken | undefined>(vibeDashboardEnvelope.resumeToken);

  const ready = session.status === "ready" && !!session.user && state.hydrated && state.snapshot;

  useEffect(() => {
    const baseUrl = process.env.NEXT_PUBLIC_UI_API ?? "http://localhost:8787";
    const schemaClient = createSchemaClient(baseUrl);
    schemaClient
      .fetchPage("vibe-kanban")
      .then((payload) => {
        setEnvelope(payload);
        setResumeToken(payload.resumeToken ?? undefined);
      })
      .catch(() => {
        // Fallback to local sample envelope
      });
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") return;
    const endpoint = process.env.NEXT_PUBLIC_WORKFLOW_STREAM ?? "ws://localhost:8787/ui/session";
    const client = new SessionContinuityClient({ workflowEndpoint: endpoint });
    client.on("workflow:update", (event) => {
      if (event.eventType === "workflow/state" && event.payload?.state === "completed") {
        state.refreshBoard().catch((error) => console.error("Failed to refresh board", error));
      }
      if (event.eventType === "workflow/stage" && event.payload?.resumeToken) {
        setResumeToken(event.payload.resumeToken as ResumeToken);
      }
    });
    client.on("workflow:resume", (token) => setResumeToken(token));
    try {
      client.connectWebSocket();
    } catch (error) {
      console.warn("Unable to connect to workflow stream", error);
    }
    return () => client.disconnect();
  }, [state]);

  const schemaRenderer = useMemo(() => {
    return ready
      ? (
          <SchemaDrivenRenderer
            schema={envelope.schema}
            context={{
              resumeWorkflow: (workflowId) => {
                console.info("Requesting resume for", workflowId);
                void state.refreshBoard();
              },
              triggerEvent: (bindingId) => {
                console.info("UI event triggered", bindingId);
              },
              data: {
                boardState: state,
                session,
                resumeToken,
              },
            }}
          />
        )
      : null;
  }, [ready, envelope.schema, state, session, resumeToken]);

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
    <>
      <NotificationCenter notifications={state.notifications} onDismiss={state.dismissNotification} />
      {schemaRenderer}
    </>
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
