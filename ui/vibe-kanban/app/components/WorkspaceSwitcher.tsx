"use client";

import { useState } from "react";

import type { BoardState } from "./useBoardState";

export function WorkspaceSwitcher({ state }: { state: BoardState }) {
  const [creating, setCreating] = useState(false);
  const [newBoardName, setNewBoardName] = useState("New concept board");

  return (
    <div className="rounded-3xl border border-white/10 bg-surface/70 p-5 text-white/70">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-sm font-semibold uppercase tracking-[0.3em] text-white/50">
            Workspace boards
          </h3>
          <p className="mt-1 text-xs text-white/40">
            Switch projects or spin up new canvases instantly.
          </p>
        </div>
        <span className="rounded-full border border-white/10 bg-white/5 px-3 py-1 text-[11px] uppercase tracking-[0.2em] text-white/50">
          {state.workspace?.billingPlan ?? "starter"}
        </span>
      </div>
      <div className="mt-4 space-y-3">
        <select
          value={state.workspaceId ?? ""}
          onChange={(event) => state.setWorkspaceId(event.target.value)}
          className="w-full rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm text-white focus:outline-none"
        >
          {state.workspaces.map((workspace) => (
            <option key={workspace.id} value={workspace.id} className="bg-surface text-white">
              {workspace.name}
            </option>
          ))}
        </select>
        <div className="space-y-2">
          {state.workspace?.boards.map((board) => (
            <button
              key={board.id}
              onClick={() => state.setBoardId(board.id)}
              className={`w-full rounded-2xl border px-4 py-3 text-left text-sm transition ${
                state.boardId === board.id
                  ? "border-indigo-400/40 bg-indigo-500/10 text-white"
                  : "border-white/10 bg-white/5 text-white/60 hover:border-white/20 hover:text-white"
              }`}
            >
              <div className="font-semibold">{board.projectName}</div>
              <p className="text-xs text-white/40">
                Updated {new Date(board.lastUpdated).toLocaleTimeString()}
              </p>
            </button>
          ))}
        </div>
      </div>
      <div className="mt-4 rounded-2xl border border-dashed border-white/20 p-4">
        {creating ? (
          <form
            className="flex flex-col gap-3"
            onSubmit={async (event) => {
              event.preventDefault();
              await state.createBoard(newBoardName);
              setCreating(false);
            }}
          >
            <input
              value={newBoardName}
              onChange={(event) => setNewBoardName(event.target.value)}
              className="rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-sm text-white focus:outline-none"
            />
            <div className="flex gap-2">
              <button
                type="submit"
                className="flex-1 rounded-full border border-indigo-400/40 bg-indigo-500/10 px-3 py-2 text-xs font-semibold text-indigo-100 transition hover:border-indigo-300/60 hover:bg-indigo-500/20"
              >
                Launch board
              </button>
              <button
                type="button"
                onClick={() => setCreating(false)}
                className="rounded-full border border-white/10 bg-white/5 px-3 py-2 text-xs text-white/60"
              >
                Cancel
              </button>
            </div>
          </form>
        ) : (
          <button
            onClick={() => setCreating(true)}
            className="w-full rounded-full border border-white/10 bg-white/5 px-4 py-2 text-sm font-semibold text-white/70 transition hover:border-white/20 hover:text-white"
          >
            + New board
          </button>
        )}
      </div>
    </div>
  );
}
