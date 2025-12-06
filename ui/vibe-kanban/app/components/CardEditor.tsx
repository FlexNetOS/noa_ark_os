"use client";

import { useEffect, useState } from "react";

import type { Goal } from "./board-types";
import { MOOD_OPTIONS } from "./moods";

type CardEditorProps = {
  goal: Goal | null;
  columnId: string | null;
  onClose: () => void;
  onUpdate: (columnId: string, goalId: string, patch: Partial<Goal>) => void;
  onDelete: (columnId: string, goalId: string) => void;
};

export function CardEditor({ goal, columnId, onClose, onUpdate, onDelete }: CardEditorProps) {
  const [title, setTitle] = useState(goal?.title ?? "");
  const [notes, setNotes] = useState(goal?.notes ?? "");
  const [mood, setMood] = useState<Goal["mood"]>(goal?.mood ?? "focus");
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    if (!goal || !columnId) {
      setVisible(false);
      return;
    }

    setTitle(goal.title);
    setNotes(goal.notes);
    setMood(goal.mood);
    setVisible(true);
  }, [goal, columnId]);

  if (!goal || !columnId) {
    return null;
  }

  const handleSave = () => {
    onUpdate(columnId, goal.id, { title, notes, mood });
    onClose();
  };

  const handleDelete = () => {
    onDelete(columnId, goal.id);
    onClose();
  };

  return (
    <div className={`fixed inset-0 z-40 flex items-center justify-center bg-background/80 backdrop-blur-xl transition-opacity duration-300 ${visible ? "opacity-100" : "opacity-0"}`}>
      <div className={`relative w-full max-w-xl rounded-3xl border border-white/10 bg-surface/95 p-8 shadow-[0_40px_120px_-45px_rgba(99,102,241,0.75)] transition-all duration-300 ${visible ? "translate-y-0 opacity-100" : "translate-y-6 opacity-0"}`}>
        <button
          type="button"
          onClick={onClose}
          className="absolute right-6 top-6 rounded-full bg-white/10 p-2 text-white/70 transition hover:bg-white/20 hover:text-white"
        >
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" className="h-4 w-4">
            <path
              fillRule="evenodd"
              d="M10 8.586 5.293 3.879 3.879 5.293 8.586 10l-4.707 4.707 1.414 1.414L10 11.414l4.707 4.707 1.414-1.414L11.414 10l4.707-4.707-1.414-1.414L10 8.586Z"
              clipRule="evenodd"
            />
          </svg>
        </button>
        <div className="mb-6 space-y-2">
          <span className="text-xs uppercase tracking-[0.3em] text-white/40">Edit goal</span>
          <h2 className="text-2xl font-semibold text-white">{goal.title}</h2>
        </div>
        <div className="space-y-5">
          <div className="space-y-2">
            <label className="text-xs font-semibold uppercase tracking-wide text-white/50">Title</label>
            <input
              value={title}
              onChange={(event) => setTitle(event.target.value)}
              className="w-full rounded-2xl border border-white/10 bg-surface/80 px-4 py-3 text-base text-white outline-none focus:border-accent-400/60"
            />
          </div>
          <div className="space-y-2">
            <label className="text-xs font-semibold uppercase tracking-wide text-white/50">Notes</label>
            <textarea
              value={notes}
              onChange={(event) => setNotes(event.target.value)}
              rows={6}
              className="w-full rounded-2xl border border-white/10 bg-surface/80 px-4 py-3 text-sm text-white outline-none focus:border-accent-400/60"
            />
          </div>
          <div className="space-y-2">
            <label className="text-xs font-semibold uppercase tracking-wide text-white/50">Mood</label>
            <div className="flex flex-wrap gap-2">
              {MOOD_OPTIONS.map((value) => (
                <button
                  key={value}
                  type="button"
                  onClick={() => setMood(value)}
                  className={`rounded-full px-4 py-2 text-xs font-semibold uppercase tracking-wide transition ${
                    mood === value
                      ? "bg-gradient-to-r from-indigo-500 via-purple-500 to-blue-500 text-white shadow-glow"
                      : "bg-white/5 text-white/60 hover:bg-white/10 hover:text-white"
                  }`}
                >
                  {value}
                </button>
              ))}
            </div>
          </div>
        </div>
        <div className="mt-8 flex items-center justify-between">
          <button
            type="button"
            onClick={handleDelete}
            className="rounded-full bg-white/5 px-4 py-2 text-xs font-semibold uppercase tracking-wide text-red-300 transition hover:bg-red-500/20 hover:text-red-100"
          >
            Delete goal
          </button>
          <button
            type="button"
            onClick={handleSave}
            className="rounded-full bg-gradient-to-r from-indigo-500 via-purple-500 to-blue-500 px-6 py-2 text-sm font-semibold uppercase tracking-wide text-white shadow-glow transition hover:shadow-[0_18px_40px_-22px_rgba(99,102,241,0.95)]"
          >
            Save goal
          </button>
        </div>
      </div>
    </div>
  );
}
