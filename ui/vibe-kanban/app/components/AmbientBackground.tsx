"use client";

export function AmbientBackground() {
  return (
    <div className="pointer-events-none fixed inset-0 -z-10 overflow-hidden">
      <div className="ambient-fade absolute inset-x-10 top-10 h-[480px] rounded-[3rem] bg-gradient-to-r from-indigo-500/35 via-purple-500/25 to-sky-500/30 blur-3xl" />
      <div className="ambient-shift absolute inset-0 bg-[radial-gradient(circle_at_top,_rgba(99,102,241,0.18),transparent_60%)]" />
    </div>
  );
}
