"use client";

import { BoardShell } from "./components/BoardShell";
import { useBoardState } from "./components/useBoardState";

export default function Page() {
  const state = useBoardState();

  if (!state.hydrated) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-background">
        <div className="relative flex items-center gap-3 rounded-full border border-white/10 bg-surface/80 px-6 py-3 text-sm font-medium text-white/60">
          <span className="h-2.5 w-2.5 animate-pulse rounded-full bg-gradient-to-r from-indigo-500 via-purple-500 to-blue-500" />
          Loading your vibe workspace…
        </div>
      </div>
    );
  }

  return <BoardShell state={state} />;
}
