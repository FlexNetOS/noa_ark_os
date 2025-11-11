"use client";

type AddColumnButtonProps = {
  onClick: () => void;
};

export function AddColumnButton({ onClick }: AddColumnButtonProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-5 py-2 text-sm font-semibold uppercase tracking-wide text-white/70 transition hover:border-white/20 hover:bg-white/15 hover:text-white"
    >
      <span className="flex h-6 w-6 items-center justify-center rounded-full bg-white/10 text-lg font-semibold">+</span>
      Add column
    </button>
  );
}
