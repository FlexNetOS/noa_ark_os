"use client";
/**
 * Renders an AI assist button that calls the backend to generate engineer prompts.
 * Embedded inside Kanban cards and orchestrates loading, preview, and copy states.
 */

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { ensureTraceId, logError } from "@noa-ark/shared-ui/logging";

import type { VibeCard } from "../../components/board-types";
import { buildPromptPayload } from "./cardPrompt";

type FetchState = {
  prompt: string | null;
  completion: string | null;
  loading: boolean;
  error: string | null;
  copied: boolean;
  showPreview: boolean;
};

type AIPromptButtonProps = {
  card: VibeCard;
};

export function AIPromptButton({ card }: AIPromptButtonProps) {
  const traceIdRef = useRef<string>(ensureTraceId());
  const [state, setState] = useState<FetchState>({
    prompt: null,
    completion: null,
    loading: false,
    error: null,
    copied: false,
    showPreview: false,
  });

  useEffect(() => {
    if (!state.copied) {
      return;
    }
    const timeout = setTimeout(() => {
      setState((current) => ({ ...current, copied: false }));
    }, 2000);
    return () => clearTimeout(timeout);
  }, [state.copied]);

  const payload = useMemo(() => buildPromptPayload(card), [card]);

  const fetchPrompt = useCallback(async () => {
    setState((current) => ({ ...current, loading: true, error: null, copied: false }));
    try {
      const response = await fetch("/api/ai/prompt", {
        method: "POST",
        headers: {
          "content-type": "application/json",
        },
        body: JSON.stringify(payload),
      });

      const data = (await response.json().catch(() => ({}))) as { prompt?: string; completion?: string | null; error?: string };
      if (!response.ok || !data.prompt) {
        throw new Error(data.error ?? `Request failed with status ${response.status}`);
      }
      setState((current) => ({
        ...current,
        prompt: data.prompt ?? null,
        completion: data.completion ?? null,
        loading: false,
        showPreview: true,
      }));
    } catch (error) {
      logError({
        component: "ai.prompt_button",
        event: "prompt_generation_failed",
        message: "Failed to generate AI prompt",
        outcome: "failure",
        traceId: traceIdRef.current,
        context: { cardId: card.id },
        error,
      });
      setState((current) => ({
        ...current,
        loading: false,
        error: error instanceof Error ? error.message : "Failed to reach AI service.",
      }));
    }
  }, [card.id, payload]);

  const copyPrompt = useCallback(async () => {
    if (!state.prompt) {
      return;
    }
    try {
      await navigator.clipboard.writeText(state.prompt);
      setState((current) => ({ ...current, copied: true }));
    } catch (error) {
      logError({
        component: "ai.prompt_button",
        event: "prompt_copy_failed",
        message: "Failed to copy generated prompt",
        outcome: "failure",
        traceId: traceIdRef.current,
        context: { cardId: card.id },
        error,
      });
      setState((current) => ({ ...current, error: "Unable to copy prompt. Please copy manually." }));
    }
  }, [card.id, state.prompt]);

  const togglePreview = useCallback(() => {
    setState((current) => ({ ...current, showPreview: !current.showPreview }));
  }, []);

  return (
    <div className="flex flex-col items-end gap-2 text-xs text-white/60">
      <div className="flex items-center gap-2">
        <button
          type="button"
          aria-label="Generate AI implementation prompt"
          className="flex h-8 w-8 items-center justify-center rounded-full border border-white/10 bg-black/40 text-white/80 transition hover:border-accent-400 hover:text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-300"
          onClick={fetchPrompt}
          disabled={state.loading}
        >
          {state.loading ? (
            <svg className="h-4 w-4 animate-spin" viewBox="0 0 24 24" role="status" aria-hidden="true">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
            </svg>
          ) : (
            <svg className="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" aria-hidden="true">
              <path d="M12 3v3m0 12v3m9-9h-3M6 12H3m14.95-6.95l-2.12 2.12M8.17 15.83l-2.12 2.12m0-12.72l2.12 2.12m6.66 6.66l2.12 2.12" />
            </svg>
          )}
        </button>
        {state.prompt ? (
          <>
            <button
              type="button"
              className="rounded-full border border-white/10 px-3 py-1 text-white/80 transition hover:border-accent-400 hover:text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-300"
              onClick={togglePreview}
            >
              {state.showPreview ? "Hide" : "Preview"}
            </button>
            <button
              type="button"
              className="rounded-full border border-white/10 px-3 py-1 text-white/80 transition hover:border-accent-400 hover:text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-300"
              onClick={copyPrompt}
            >
              Copy
            </button>
          </>
        ) : null}
      </div>
      {state.error ? (
        <p role="alert" className="max-w-[16rem] text-right text-red-300">
          {state.error}
        </p>
      ) : null}
      {state.copied ? (
        <p aria-live="polite" className="text-emerald-300">
          Prompt copied to clipboard.
        </p>
      ) : null}
      {state.prompt && state.showPreview ? (
        <pre className="max-h-64 w-full overflow-auto rounded-lg border border-white/10 bg-black/50 p-3 text-left text-[11px] leading-relaxed text-white/80">
          {state.prompt}
        </pre>
      ) : null}
    </div>
  );
}
