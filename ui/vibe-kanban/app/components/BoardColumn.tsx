"use client";

import { useState } from "react";
import type { DragEvent as ReactDragEvent } from "react";

import type { Goal, VibeColumn } from "./board-types";
import { KanbanCard } from "./KanbanCard";
import { AddCardComposer } from "./AddCardComposer";
import { DRAG_DATA_TYPE } from "./drag-utils";

const accentClassName = (accent: string) => `h-10 w-10 rounded-2xl bg-gradient-to-br p-[2px] ${accent}`;

type BoardColumnProps = {
  column: VibeColumn;
  onRemove: (id: string) => void;
  onRename: (id: string, title: string) => void;
  onAddGoal: (title: string, notes?: string) => void;
  onGoalOpen: (goal: Goal) => void;
  enableComposer: boolean;
  onGoalDragStart: (goal: Goal, event: ReactDragEvent<HTMLButtonElement>) => void;
  onGoalDragOver: (goalId: string, event: ReactDragEvent<HTMLButtonElement>) => void;
  onGoalDragLeave: (goalId: string) => void;
  onGoalDrop: (goalId: string, event: ReactDragEvent<HTMLButtonElement>) => void;
  onGoalDragEnd: () => void;
  onColumnDragStart: (event: ReactDragEvent<HTMLButtonElement>) => void;
  onColumnDragEnd: () => void;
  onColumnSurfaceDragOver: (event: ReactDragEvent<HTMLDivElement>) => void;
  onColumnSurfaceDragLeave: () => void;
  onColumnSurfaceDrop: (event: ReactDragEvent<HTMLDivElement>) => void;
  isDraggingColumn: boolean;
  isColumnDropTarget: boolean;
  isGoalDropZoneActive: boolean;
  draggingGoalId: string | null;
  dropTargetGoalId: string | null;
};

export function BoardColumn({
  column,
  onRemove,
  onRename,
  onAddGoal,
  onGoalOpen,
  enableComposer,
  onGoalDragStart,
  onGoalDragOver,
  onGoalDragLeave,
  onGoalDrop,
  onGoalDragEnd,
  onColumnDragStart,
  onColumnDragEnd,
  onColumnSurfaceDragOver,
  onColumnSurfaceDragLeave,
  onColumnSurfaceDrop,
  isDraggingColumn,
  isColumnDropTarget,
  isGoalDropZoneActive,
  draggingGoalId,
  dropTargetGoalId,
}: BoardColumnProps) {
  const [isHoveringSurface, setIsHoveringSurface] = useState(false);

  const handleSurfaceDragOver = (event: ReactDragEvent<HTMLDivElement>) => {
    if (!event.dataTransfer?.types.includes(DRAG_DATA_TYPE)) return;
    onColumnSurfaceDragOver(event);
    setIsHoveringSurface(true);
  };

  const handleSurfaceDragLeave = (event: ReactDragEvent<HTMLDivElement>) => {
    const nextTarget = event.relatedTarget as Node | null;
    if (nextTarget && event.currentTarget.contains(nextTarget)) {
      return;
    }
    setIsHoveringSurface(false);
    onColumnSurfaceDragLeave();
  };

  const handleSurfaceDrop = (event: ReactDragEvent<HTMLDivElement>) => {
    setIsHoveringSurface(false);
    onColumnSurfaceDrop(event);
  };

  const columnClasses = [
    "relative flex h-full w-80 shrink-0 flex-col rounded-3xl border border-white/5 bg-surface/90 p-5 backdrop-blur-xl transition-all duration-200",
    "shadow-card",
  ];

  if (isDraggingColumn) {
    columnClasses.push("border-accent-400/60 shadow-glow scale-[1.02]");
  }

  if (isColumnDropTarget) {
    columnClasses.push("border-accent-300/80 shadow-glow");
  }

  if (isGoalDropZoneActive || isHoveringSurface) {
    columnClasses.push("ring-2 ring-accent-300/40 ring-offset-2 ring-offset-surface");
  }

  return (
    <div
      className={columnClasses.join(" ")}
      onDragOver={handleSurfaceDragOver}
      onDragLeave={handleSurfaceDragLeave}
      onDrop={handleSurfaceDrop}
    >
      <div className="flex items-center justify-between">
        <div className="flex flex-1 items-center gap-3">
          <button
            type="button"
            aria-label="Drag column"
            className="flex h-10 w-10 items-center justify-center rounded-2xl border border-white/5 bg-white/5 text-white/40 transition hover:border-white/20 hover:text-white"
            draggable
            onDragStart={onColumnDragStart}
            onDragEnd={onColumnDragEnd}
          >
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" className="h-4 w-4">
              <path d="M7 4.5a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0Zm0 11a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0Zm9-11a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0Zm-9 5.5a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0Zm9 0a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0Zm-9 5.5a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0Zm9 0a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0Z" />
            </svg>
          </button>
          <span className={accentClassName(column.accent)}>
            <span className="flex h-full w-full items-center justify-center rounded-[1rem] bg-surface/90 text-sm font-semibold uppercase tracking-wide text-white/80">
              {column.title.slice(0, 2)}
            </span>
          </span>
          <input
            defaultValue={column.title}
            onBlur={(event) => onRename(column.id, event.currentTarget.value)}
            className="w-full bg-transparent text-lg font-semibold text-white outline-none placeholder:text-white/40"
          />
        </div>
        <button
          type="button"
          onClick={() => onRemove(column.id)}
          className="rounded-full bg-white/5 p-2 text-white/60 transition hover:bg-white/10 hover:text-white"
          aria-label="Remove column"
        >
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" className="h-4 w-4">
            <path
              fillRule="evenodd"
              d="M10 8.586 5.293 3.879 3.879 5.293 8.586 10l-4.707 4.707 1.414 1.414L10 11.414l4.707 4.707 1.414-1.414L11.414 10l4.707-4.707-1.414-1.414L10 8.586Z"
              clipRule="evenodd"
            />
          </svg>
        </button>
      </div>
      <div className="mt-6 flex flex-1 flex-col gap-4 overflow-y-auto pr-2">
        <div className="flex flex-col gap-4">
          {column.goals.map((goal) => (
            <KanbanCard
              key={goal.id}
              goal={goal}
              onOpen={onGoalOpen}
              onDragStart={(event) => onGoalDragStart(goal, event)}
              onDragOver={(event) => onGoalDragOver(goal.id, event)}
              onDragLeave={() => onGoalDragLeave(goal.id)}
              onDrop={(event) => onGoalDrop(goal.id, event)}
              onDragEnd={onGoalDragEnd}
              isDragging={draggingGoalId === goal.id}
              isDropTarget={dropTargetGoalId === goal.id}
            />
          ))}
          {enableComposer && <AddCardComposer onSubmit={onAddGoal} />}
        </div>
      </div>
      <div className="pointer-events-none absolute inset-0 rounded-3xl border border-white/5" />
    </div>
  );
}
