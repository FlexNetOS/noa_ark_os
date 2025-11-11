"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import type { BoardSnapshot, VibeCard, VibeColumn } from "./board-types";

const STORAGE_KEY = "vibe-kanban-board";

const accentPalette = [
  "from-indigo-500 via-purple-500 to-blue-500",
  "from-sky-500 via-cyan-400 to-emerald-400",
  "from-rose-500 via-pink-500 to-amber-400",
  "from-violet-500 via-indigo-400 to-fuchsia-500",
  "from-emerald-500 via-lime-400 to-teal-400"
];

const defaultSnapshot: BoardSnapshot = {
  projectName: "Vibe Coding Launchpad",
  lastUpdated: new Date().toISOString(),
  columns: [
    {
      id: "todo",
      title: "To Do",
      accent: accentPalette[0],
      cards: [
        {
          id: "card-1",
          title: "Ideate hero motion",
          notes: "Sketch the flowing hero animation that loops smoothly across the dashboard.",
          createdAt: new Date().toISOString(),
          mood: "flow"
        },
        {
          id: "card-2",
          title: "Sound-reactive palette",
          notes: "Research how to tie track BPM to gradient shifts on the home screen.",
          createdAt: new Date().toISOString(),
          mood: "focus"
        }
      ]
    },
    {
      id: "in-progress",
      title: "In Progress",
      accent: accentPalette[1],
      cards: [
        {
          id: "card-3",
          title: "Prototype kanban drag",
          notes: "Polish easing curve + inertia for drag transitions.",
          createdAt: new Date().toISOString(),
          mood: "hype"
        }
      ]
    },
    {
      id: "done",
      title: "Completed",
      accent: accentPalette[3],
      cards: [
        {
          id: "card-4",
          title: "Mood-driven theme",
          notes: "Shipped gradient system that syncs with vibes.",
          createdAt: new Date().toISOString(),
          mood: "chill"
        }
      ]
    }
  ]
};

function hydrateSnapshot(raw: string | null): BoardSnapshot {
  if (!raw) return defaultSnapshot;
  try {
    const parsed = JSON.parse(raw) as BoardSnapshot;
    if (!parsed.columns || !Array.isArray(parsed.columns)) {
      return defaultSnapshot;
    }
    return {
      ...parsed,
      lastUpdated: parsed.lastUpdated ?? new Date().toISOString(),
    };
  } catch (error) {
    console.warn("Failed to parse stored board snapshot", error);
    return defaultSnapshot;
  }
}

function createCard(partial: Partial<VibeCard> & { title: string }): VibeCard {
  const id =
    partial.id ?? (typeof crypto !== "undefined" && crypto.randomUUID ? crypto.randomUUID() : `card-${Date.now()}`);
  return {
    id,
    title: partial.title,
    notes: partial.notes ?? "",
    mood: partial.mood ?? "focus",
    createdAt: partial.createdAt ?? new Date().toISOString(),
  };
}

function arrayMove<T>(list: T[], from: number, to: number) {
  if (from === to) return list;
  const next = [...list];
  const [item] = next.splice(from, 1);
  next.splice(to, 0, item);
  return next;
}

export function useBoardState() {
  const [snapshot, setSnapshot] = useState<BoardSnapshot>(defaultSnapshot);
  const [hydrated, setHydrated] = useState(false);

  useEffect(() => {
    if (typeof window === "undefined") return;
    const stored = window.localStorage.getItem(STORAGE_KEY);
    setSnapshot(hydrateSnapshot(stored));
    setHydrated(true);
  }, []);

  useEffect(() => {
    if (!hydrated || typeof window === "undefined") return;
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshot));
  }, [snapshot, hydrated]);

  const updateColumns = useCallback(
    (updater: (columns: VibeColumn[]) => VibeColumn[], updateTimestamp = true) => {
      setSnapshot((prev) => ({
        ...prev,
        columns: updater(prev.columns),
        lastUpdated: updateTimestamp ? new Date().toISOString() : prev.lastUpdated,
      }));
    },
    []
  );

  const columnCount = snapshot.columns.length;

  const addColumn = useCallback(
    (title: string) => {
      const accent = accentPalette[columnCount % accentPalette.length];
      updateColumns((columns) => [
        ...columns,
        { id: `column-${Date.now()}`, title: title.trim() || "Untitled", accent, cards: [] },
      ]);
    },
    [columnCount, updateColumns]
  );

  const removeColumn = useCallback(
    (columnId: string) => {
      updateColumns((columns) => columns.filter((col) => col.id !== columnId));
    },
    [updateColumns]
  );

  const renameColumn = useCallback(
    (columnId: string, title: string) => {
      updateColumns((columns) =>
        columns.map((column) => (column.id === columnId ? { ...column, title: title.trim() || column.title } : column))
      );
    },
    [updateColumns]
  );

  const addCard = useCallback(
    (columnId: string, title: string, notes = "") => {
      updateColumns((columns) =>
        columns.map((column) =>
          column.id === columnId
            ? { ...column, cards: [...column.cards, createCard({ title, notes })] }
            : column
        )
      );
    },
    [updateColumns]
  );

  const updateCard = useCallback(
    (columnId: string, cardId: string, patch: Partial<VibeCard>) => {
      updateColumns((columns) =>
        columns.map((column) =>
          column.id === columnId
            ? {
                ...column,
                cards: column.cards.map((card) =>
                  card.id === cardId ? { ...card, ...patch, title: patch.title?.trim() || card.title } : card
                ),
              }
            : column
        )
      );
    },
    [updateColumns]
  );

  const removeCard = useCallback(
    (columnId: string, cardId: string) => {
      updateColumns((columns) =>
        columns.map((column) =>
          column.id === columnId
            ? { ...column, cards: column.cards.filter((card) => card.id !== cardId) }
            : column
        )
      );
    },
    [updateColumns]
  );

  const moveColumn = useCallback(
    (activeId: string, overId: string) => {
      if (activeId === overId) return;
      updateColumns((columns) => {
        const oldIndex = columns.findIndex((column) => column.id === activeId);
        const newIndex = columns.findIndex((column) => column.id === overId);
        if (oldIndex === -1 || newIndex === -1) return columns;
        return arrayMove(columns, oldIndex, newIndex);
      });
    },
    [updateColumns]
  );

  const moveCardWithinColumn = useCallback(
    (columnId: string, activeId: string, overId: string) => {
      if (activeId === overId) return;
      updateColumns((columns) =>
        columns.map((column) => {
          if (column.id !== columnId) return column;
          const oldIndex = column.cards.findIndex((card) => card.id === activeId);
          const newIndex = column.cards.findIndex((card) => card.id === overId);
          if (oldIndex === -1 || newIndex === -1) return column;
          return {
            ...column,
            cards: arrayMove(column.cards, oldIndex, newIndex),
          };
        })
      );
    },
    [updateColumns]
  );

  const moveCardToColumn = useCallback(
    (activeColumnId: string, overColumnId: string, activeCardId: string, overCardId?: string) => {
      if (activeColumnId === overColumnId) return;
      updateColumns((columns) => {
        const sourceIndex = columns.findIndex((column) => column.id === activeColumnId);
        const targetIndex = columns.findIndex((column) => column.id === overColumnId);
        if (sourceIndex === -1 || targetIndex === -1) return columns;

        const sourceColumn = columns[sourceIndex];
        const targetColumn = columns[targetIndex];
        const activeCardIndex = sourceColumn.cards.findIndex((card) => card.id === activeCardId);
        if (activeCardIndex === -1) return columns;

        const updatedSourceCards = [...sourceColumn.cards];
        const [movedCard] = updatedSourceCards.splice(activeCardIndex, 1);

        const updatedTargetCards = [...targetColumn.cards];
        const insertAt = overCardId
          ? updatedTargetCards.findIndex((card) => card.id === overCardId)
          : updatedTargetCards.length;
        const nextCards = [...updatedTargetCards];
        nextCards.splice(insertAt === -1 ? nextCards.length : insertAt, 0, movedCard);

        const updatedColumns = [...columns];
        updatedColumns[sourceIndex] = { ...sourceColumn, cards: updatedSourceCards };
        updatedColumns[targetIndex] = { ...targetColumn, cards: nextCards };
        return updatedColumns;
      });
    },
    [updateColumns]
  );

  const setProjectName = useCallback((projectName: string) => {
    setSnapshot((prev) => ({ ...prev, projectName: projectName.trim() || prev.projectName }));
  }, []);

  const context = useMemo(
    () => ({
      snapshot,
      hydrated,
      addColumn,
      removeColumn,
      renameColumn,
      addCard,
      updateCard,
      removeCard,
      moveCardWithinColumn,
      moveCardToColumn,
      moveColumn,
      setProjectName,
    }),
    [
      snapshot,
      hydrated,
      addColumn,
      removeColumn,
      renameColumn,
      addCard,
      updateCard,
      removeCard,
      moveCardWithinColumn,
      moveCardToColumn,
      moveColumn,
      setProjectName,
    ]
  );

  return context;
}

export type BoardState = ReturnType<typeof useBoardState>;
