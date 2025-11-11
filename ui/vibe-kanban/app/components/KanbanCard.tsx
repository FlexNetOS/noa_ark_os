"use client";

import { useState } from "react";
import type { DragEvent as ReactDragEvent } from "react";

import type { VibeCard } from "./board-types";
import { CardMoodBadge } from "./CardMoodBadge";

type KanbanCardProps = {
  card: VibeCard;
  onOpen: (card: VibeCard) => void;
  onDragStart: (event: ReactDragEvent<HTMLButtonElement>) => void;
  onDragOver: (event: ReactDragEvent<HTMLButtonElement>) => void;
  onDragLeave: () => void;
  onDrop: (event: ReactDragEvent<HTMLButtonElement>) => void;
  onDragEnd: () => void;
  isDragging: boolean;
  isDropTarget: boolean;
};

export function KanbanCard({
  card,
  onOpen,
  onDragStart,
  onDragOver,
  onDragLeave,
  onDrop,
  onDragEnd,
  isDragging,
  isDropTarget,
}: KanbanCardProps) {
  const [isHovered, setIsHovered] = useState(false);

  const classes = [
    "group flex w-full flex-col items-stretch rounded-2xl border border-white/5 bg-gradient-to-br from-white/5 via-white/0 to-white/10 p-4 text-left transition",
    "hover:border-white/20 hover:bg-white/10",
  ];

  if (isDragging) {
    classes.push("border-accent-400/70 shadow-glow scale-[1.01]");
  }

  if (isDropTarget) {
    classes.push("border-accent-300/70 shadow-glow");
  }

  if (isHovered && !isDragging) {
    classes.push("bg-white/10 border-white/20");
  }

  return (
    <button
      type="button"
      draggable
      onClick={() => onOpen(card)}
      onDragStart={(event) => {
        onDragStart(event);
      }}
      onDragOver={(event) => {
        onDragOver(event);
      }}
      onDragLeave={() => {
        setIsHovered(false);
        onDragLeave();
      }}
      onDrop={(event) => {
        setIsHovered(false);
        onDrop(event);
      }}
      onDragEnter={() => setIsHovered(true)}
      onDragEnd={() => {
        setIsHovered(false);
        onDragEnd();
      }}
      className={classes.join(" ")}
    >
      <div className="flex items-center justify-between">
        <h3 className="text-base font-semibold text-white/90 group-hover:text-white">{card.title}</h3>
        <CardMoodBadge mood={card.mood} />
      </div>
      {card.notes && <p className="mt-3 line-clamp-3 text-sm text-white/60">{card.notes}</p>}
      <div className="mt-4 flex items-center justify-between text-xs text-white/40">
        <span>Created {new Date(card.createdAt).toLocaleDateString()}</span>
        <span className="flex items-center gap-1 text-white/50">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            className="h-4 w-4"
          >
            <path d="M3 5h18M6 5v14a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V5" />
            <path d="M9 5V3h6v2" />
          </svg>
          tap to open
        </span>
      </div>
    </button>
  );
}
