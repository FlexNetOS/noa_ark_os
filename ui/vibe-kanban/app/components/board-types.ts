export type VibeCard = {
  id: string;
  title: string;
  notes: string;
  createdAt: string;
  mood: "focus" | "flow" | "chill" | "hype";
};

export type VibeColumn = {
  id: string;
  title: string;
  accent: string;
  cards: VibeCard[];
};

export type BoardSnapshot = {
  columns: VibeColumn[];
  lastUpdated: string;
  projectName: string;
};
