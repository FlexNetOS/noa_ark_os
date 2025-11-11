import type { VibeCard } from "./board-types";
import { MOOD_STYLES } from "./moods";

export function CardMoodBadge({ mood }: { mood: VibeCard["mood"] }) {
  const classes = `inline-flex items-center gap-1 rounded-full px-2.5 py-1 text-xs font-medium tracking-wide uppercase ${MOOD_STYLES[mood]}`;

  return (
    <span className={classes}>
      <span className="h-1.5 w-1.5 rounded-full bg-current" />
      {mood}
    </span>
  );
}
