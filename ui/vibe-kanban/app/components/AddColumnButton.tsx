"use client";

type AddColumnButtonProps = {
  onClick: () => void;
  disabled?: boolean;
  disabledReason?: string;
};

export function AddColumnButton({
  onClick,
  disabled = false,
  disabledReason,
}: AddColumnButtonProps) {
  const classes = [
    "inline-flex items-center gap-2 rounded-full border px-5 py-2 text-sm font-semibold uppercase tracking-wide transition",
  ];

  if (disabled) {
    classes.push("border-white/10 bg-white/5 text-white/40 opacity-60 cursor-not-allowed");
  } else {
    classes.push(
      "border-white/10 bg-white/5 text-white/70 hover:border-white/20 hover:bg-white/15 hover:text-white",
    );
  }

  return (
    <button
      type="button"
      onClick={onClick}
      className={classes.join(" ")}
      disabled={disabled}
      aria-disabled={disabled}
      title={disabled ? disabledReason ?? "Capability unavailable" : undefined}
    >
      <span className="flex h-6 w-6 items-center justify-center rounded-full bg-white/10 text-lg font-semibold">
        +
      </span>
      Add column
    </button>
  );
}
