import type { Goal } from "./board-types";

export const MOOD_STYLES: Record<Goal["mood"], string> = {
  focus: "bg-indigo-500/20 text-indigo-200 border border-indigo-500/30",
  flow: "bg-cyan-500/20 text-cyan-100 border border-cyan-500/30",
  chill: "bg-emerald-500/20 text-emerald-100 border border-emerald-500/30",
  hype: "bg-fuchsia-500/20 text-fuchsia-100 border border-fuchsia-500/30",
};

export const MOOD_OPTIONS = Object.keys(MOOD_STYLES) as Array<Goal["mood"]>;
