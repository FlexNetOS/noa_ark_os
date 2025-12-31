"use client";

import { useCallback, useMemo, useState } from "react";

import type { BoardState } from "./useBoardState";

const dropTypeOptions = [
  { value: "repos", label: "External Repo" },
  { value: "forks", label: "Fork" },
  { value: "mirrors", label: "Mirror" },
  { value: "stale", label: "Stale Codebase" },
  { value: "internal", label: "Internal Asset" },
];

type UploadPanelProps = {
  state: BoardState;
};

export function UploadPanel({ state }: UploadPanelProps) {
  const [dropType, setDropType] = useState<string>(dropTypeOptions[0]?.value ?? "repos");
  const [isUploading, setUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const recentReceipts = useMemo(() => state.uploadReceipts.slice(0, 3), [state.uploadReceipts]);

  const handleSubmit = useCallback(
    async (event: React.FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      setError(null);
      const formData = new FormData(event.currentTarget);
      const file = formData.get("file");
      if (!(file instanceof File) || !file.size) {
        setError("Select a file to upload");
        return;
      }
      setUploading(true);
      try {
        await state.uploadArtifact({ file, dropType });
        event.currentTarget.reset();
      } catch (uploadError) {
        setError(uploadError instanceof Error ? uploadError.message : "Failed to upload artifact");
      } finally {
        setUploading(false);
      }
    },
    [dropType, state]
  );

  return (
    <div className="rounded-3xl border border-white/10 bg-surface/70 p-5 text-white/70">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h3 className="text-sm font-semibold uppercase tracking-[0.3em] text-white/50">CRC Uploads</h3>
          <p className="mt-1 text-xs text-white/40">
            Ship artifacts into the CRC drop-in. Hashes are recorded in the workspace notifications stream.
          </p>
        </div>
      </div>
      <form className="mt-4 space-y-3" onSubmit={handleSubmit}>
        <div className="grid grid-cols-1 gap-3 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
          <label className="flex flex-col gap-2 text-xs uppercase tracking-[0.2em] text-white/40">
            Drop Type
            <select
              name="dropType"
              value={dropType}
              onChange={(event) => setDropType(event.target.value)}
              className="rounded-2xl border border-white/10 bg-white/5 px-3 py-2 text-sm text-white focus:outline-none"
            >
              {dropTypeOptions.map((option) => (
                <option key={option.value} value={option.value} className="bg-surface text-white">
                  {option.label}
                </option>
              ))}
            </select>
          </label>
          <label className="flex flex-col gap-2 text-xs uppercase tracking-[0.2em] text-white/40">
            Artifact File
            <input
              type="file"
              name="file"
              className="rounded-2xl border border-white/10 bg-white/5 px-3 py-2 text-sm text-white focus:outline-none"
            />
          </label>
        </div>
        {error && <p className="text-xs text-rose-300">{error}</p>}
        <div className="flex items-center justify-between gap-3">
          <button
            type="submit"
            disabled={isUploading}
            className="rounded-full border border-emerald-400/40 bg-emerald-500/10 px-4 py-2 text-xs font-semibold text-emerald-200 transition hover:border-emerald-300/60 hover:bg-emerald-500/20 disabled:cursor-not-allowed disabled:border-white/10 disabled:bg-white/5 disabled:text-white/30"
          >
            {isUploading ? "Uploading…" : "Send to CRC"}
          </button>
          {recentReceipts.length > 0 && (
            <div className="text-right text-[11px] uppercase tracking-[0.2em] text-white/40">
              <p>Recent receipts</p>
              <ul className="mt-1 space-y-1 text-[10px] normal-case text-white/60">
                {recentReceipts.map((receipt) => (
                  <li key={receipt.id} className="truncate">
                    <span className="font-mono text-[10px] text-white/70">{receipt.casKeys[0]}</span>
                    {receipt.casKeys.length > 1 ? ` (+${receipt.casKeys.length - 1})` : ""}
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      </form>
    </div>
  );
}
