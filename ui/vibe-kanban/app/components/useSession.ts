"use client";

import { useCallback, useEffect, useState } from "react";

export type ClientSessionUser = {
  id: string;
  name: string;
};

type SessionState = {
  status: "loading" | "ready";
  user: ClientSessionUser | null;
};

export function useSession() {
  const [state, setState] = useState<SessionState>({ status: "loading", user: null });

  useEffect(() => {
    let cancelled = false;
    fetch("/api/session", { cache: "no-store" })
      .then((response) => response.json())
      .then((payload) => {
        if (!cancelled) {
          setState({ status: "ready", user: payload.user });
        }
      })
      .catch(() => {
        if (!cancelled) {
          setState({ status: "ready", user: null });
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const signIn = useCallback(async (name: string) => {
    const response = await fetch("/api/session", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name }),
    });
    if (!response.ok) {
      throw new Error("Failed to establish session");
    }
    const payload = await response.json();
    setState({ status: "ready", user: { id: payload.id, name: payload.name } });
  }, []);

  const signOut = useCallback(async () => {
    await fetch("/api/session", { method: "DELETE" });
    setState({ status: "ready", user: null });
  }, []);

  return {
    ...state,
    signIn,
    signOut,
  };
}
