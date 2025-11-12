"use client";

import type { VibeCard } from "./board-types";
import type { BoardState } from "./useBoardState";

type AssistPanelProps = {
  assist: BoardState["assist"];
  onRequest: () => void;
};

export function AssistPanel({ assist, onRequest }: AssistPanelProps) {
  return (
    <div className="rounded-3xl border border-white/10 bg-surface/70 p-5 text-white/70">
      <div className="flex items-center justify-between gap-4">
        <div>
          <h3 className="text-sm font-semibold uppercase tracking-[0.3em] text-white/50">Agent factory</h3>
          <p className="mt-1 text-xs text-white/40">Spin up guidance, summaries, and next moves using retrieval.</p>
        </div>
        <button
          onClick={onRequest}
          className="rounded-full border border-indigo-400/40 bg-indigo-500/10 px-4 py-2 text-xs font-semibold text-indigo-200 transition hover:border-indigo-300/60 hover:bg-indigo-500/20"
        >
          Spark assist
        </button>
      </div>
      {assist ? (
        <div className="mt-4 space-y-4">
          {assist.focusCard && (
            <FocusCardCard focusCard={assist.focusCard} />
          )}
          <ul className="space-y-3">
            {assist.suggestions.map((suggestion, index) => (
              <li key={index} className="rounded-2xl border border-white/10 bg-white/5 p-4">
                <div className="text-sm font-semibold text-white">{suggestion.title}</div>
                <p className="mt-1 text-xs text-white/60">{suggestion.detail}</p>
              </li>
            ))}
          </ul>
          <p className="text-[11px] uppercase tracking-[0.2em] text-white/30">
            Refreshed {new Date(assist.updatedAt).toLocaleTimeString()}
          </p>
        </div>
      ) : (
        <p className="mt-4 text-sm text-white/40">Tap spark assist to receive next-step intelligence.</p>
      )}
    </div>
  );
}

function FocusCardCard({ focusCard }: { focusCard: VibeCard }) {
  return (
    <div className="rounded-2xl border border-amber-400/30 bg-amber-500/10 p-4 text-amber-100">
      <div className="text-xs uppercase tracking-[0.3em] text-amber-200">Spotlight card</div>
      <div className="mt-1 text-sm font-semibold text-amber-50">{focusCard.title}</div>
      <p className="mt-1 text-xs text-amber-100/70">{focusCard.notes || "No notes yet"}</p>
    </div>
  );
}
