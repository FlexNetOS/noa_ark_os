"use client";

import { useState } from "react";

type AddGoalComposerProps = {
  onSubmit: (title: string, notes?: string) => void;
};

export function AddCardComposer({ onSubmit }: AddGoalComposerProps) {
  const [open, setOpen] = useState(false);
  const [title, setTitle] = useState("");
  const [notes, setNotes] = useState("");

  const handleAdd = () => {
    if (!title.trim()) return;
    onSubmit(title, notes);
    setTitle("");
    setNotes("");
    setOpen(false);
  };

  const handleClose = () => {
    setOpen(false);
    setTitle("");
    setNotes("");
  };

  if (!open) {
    return (
      <button
        type="button"
        onClick={() => setOpen(true)}
        className="flex w-full items-center justify-center gap-2 rounded-2xl border border-dashed border-white/10 bg-white/5 px-4 py-3 text-sm font-medium text-white/70 transition hover:border-white/20 hover:bg-white/10 hover:text-white"
      >
        <span className="flex h-6 w-6 items-center justify-center rounded-full bg-white/10 text-lg font-semibold">+</span>
        Set a new goal
      </button>
    );
  }

  return (
    <div className="space-y-3 rounded-2xl border border-white/10 bg-surface/80 p-4 text-sm text-white/70 shadow-card">
      <input
        value={title}
        onChange={(event) => setTitle(event.target.value)}
        placeholder="Goal title"
        className="w-full rounded-xl border border-white/10 bg-surface/90 px-3 py-2 text-sm text-white placeholder:text-white/40 focus:border-accent-400/60 focus:outline-none"
      />
      <textarea
        value={notes}
        onChange={(event) => setNotes(event.target.value)}
        rows={3}
        placeholder="Add notes, links, or context"
        className="w-full rounded-xl border border-white/10 bg-surface/90 px-3 py-2 text-sm text-white placeholder:text-white/40 focus:border-accent-400/60 focus:outline-none"
      />
      <div className="flex items-center justify-end gap-2">
        <button
          type="button"
          onClick={handleClose}
          className="rounded-full px-3 py-1 text-xs font-medium text-white/40 transition hover:text-white/70"
        >
          Cancel
        </button>
        <button
          type="button"
          onClick={handleAdd}
          className="rounded-full bg-gradient-to-r from-indigo-500 via-purple-500 to-blue-500 px-4 py-1.5 text-xs font-semibold uppercase tracking-wide text-white shadow-glow transition hover:shadow-[0_12px_30px_-15px_rgba(99,102,241,0.85)]"
        >
          Add goal
        </button>
      </div>
    </div>
  );
}
