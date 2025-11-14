"use client";

import { useMemo, useState } from "react";
import type { DragEvent as ReactDragEvent } from "react";

import type { Goal } from "./board-types";
import type { BoardState } from "./useBoardState";
import { BoardColumn } from "./BoardColumn";
import { CardEditor } from "./CardEditor";
import { AmbientBackground } from "./AmbientBackground";
import { BoardHeader } from "./BoardHeader";
import { isFeatureEnabled } from "./featureFlags";
import { readDragData, setDragData } from "./drag-utils";

const COMPLETED_COLUMN_PATTERN = /done|complete|finished/i;

type GoalTarget = {
  columnId: string;
  goalId: string;
};

type BoardShellProps = {
  state: BoardState;
};

export function BoardShell({ state }: BoardShellProps) {
  const {
    snapshot,
    addColumn,
    removeColumn,
    renameColumn,
    addGoal,
    updateGoal,
    removeGoal,
    moveGoalWithinColumn,
    moveGoalToColumn,
    moveColumn,
    setProjectName,
  } = state;

  const [activeGoal, setActiveGoal] = useState<Goal | null>(null);
  const [activeColumnId, setActiveColumnId] = useState<string | null>(null);
  const [draggingGoalId, setDraggingGoalId] = useState<string | null>(null);
  const [draggingColumnId, setDraggingColumnId] = useState<string | null>(null);
  const [goalDropTarget, setGoalDropTarget] = useState<GoalTarget | null>(null);
  const [columnDropTarget, setColumnDropTarget] = useState<string | null>(null);
  const [columnGoalDropTarget, setColumnGoalDropTarget] = useState<string | null>(null);

  const totalGoalCount = useMemo(() => {
    if (!snapshot) return 0;
    return snapshot.columns.reduce((acc, column) => acc + column.goals.length, 0);
  }, [snapshot]);

  const completedGoalCount = useMemo(() => {
    if (!snapshot) return 0;

    return snapshot.columns.reduce((count, column) => {
      if (!COMPLETED_COLUMN_PATTERN.test(column.title)) {
        return count;
      }

      return count + column.goals.length;
    }, 0);
  }, [snapshot]);

  const ambientBackdropEnabled = isFeatureEnabled("ambientBackdrop");
  const boardMetricsEnabled =
    !state.capabilities.loading &&
    isFeatureEnabled("boardMetrics") &&
    state.capabilities.has("kanban.metrics");
  const quickComposerEnabled =
    !state.capabilities.loading &&
    isFeatureEnabled("quickComposer") &&
    state.capabilities.has("kanban.quickComposer");
  const canManageColumns = !state.capabilities.loading && state.capabilities.has("kanban.manageColumns");
  const addColumnDisabledReason = canManageColumns
    ? undefined
    : "Enable the kanban.manageColumns capability to add new columns.";

  if (!snapshot) {
    return null;
  }

  const handleGoalDragStart = (columnId: string, goal: Goal, event: ReactDragEvent<HTMLButtonElement>) => {
    setDragData(event, { type: "goal", columnId, goalId: goal.id });
    setDraggingGoalId(goal.id);
  };

  const handleGoalDragOver = (columnId: string, goalId: string, event: ReactDragEvent<HTMLButtonElement>) => {
    const payload = readDragData(event);
    if (!payload || payload.type !== "goal") return;
    event.preventDefault();
    setGoalDropTarget({ columnId, goalId });
  };

  const handleGoalDragLeave = (columnId: string, goalId: string) => {
    setGoalDropTarget((current) => {
      if (!current) return null;
      if (current.columnId === columnId && current.goalId === goalId) {
        return null;
      }
      return current;
    });
  };

  const handleGoalDrop = (columnId: string, goalId: string, event: ReactDragEvent<HTMLButtonElement>) => {
    const payload = readDragData(event);
    if (!payload || payload.type !== "goal") return;
    event.preventDefault();

    if (payload.columnId === columnId) {
      if (payload.goalId !== goalId) {
        moveGoalWithinColumn(columnId, payload.goalId, goalId);
      }
    } else {
      moveGoalToColumn(payload.columnId, columnId, payload.goalId, goalId);
    }

    setDraggingGoalId(null);
    setGoalDropTarget(null);
    setColumnGoalDropTarget(null);
  };

  const handleColumnSurfaceDragOver = (columnId: string, event: ReactDragEvent<HTMLDivElement>) => {
    const payload = readDragData(event);
    if (!payload) return;

    if (payload.type === "goal") {
      event.preventDefault();
      setColumnGoalDropTarget(columnId);
    }

    if (payload.type === "column") {
      event.preventDefault();
      setColumnDropTarget(columnId);
    }
  };

  const handleColumnSurfaceDrop = (columnId: string, event: ReactDragEvent<HTMLDivElement>) => {
    const payload = readDragData(event);
    if (!payload) return;
    event.preventDefault();

    if (payload.type === "goal") {
      if (payload.columnId !== columnId) {
        moveGoalToColumn(payload.columnId, columnId, payload.goalId);
      }
      setDraggingGoalId(null);
    }

    if (payload.type === "column") {
      if (payload.columnId !== columnId) {
        moveColumn(payload.columnId, columnId);
      }
      setDraggingColumnId(null);
    }

    setGoalDropTarget(null);
    setColumnDropTarget(null);
    setColumnGoalDropTarget(null);
  };

  const handleColumnSurfaceDragLeave = (columnId: string) => {
    setColumnDropTarget((current) => (current === columnId ? null : current));
    setColumnGoalDropTarget((current) => (current === columnId ? null : current));
  };

  const handleGoalDragEnd = () => {
    setDraggingGoalId(null);
    setGoalDropTarget(null);
    setColumnGoalDropTarget(null);
  };

  const handleColumnDragStart = (columnId: string, event: ReactDragEvent<HTMLButtonElement>) => {
    setDragData(event, { type: "column", columnId });
    setDraggingColumnId(columnId);
  };

  const handleColumnDragEnd = () => {
    setDraggingColumnId(null);
    setColumnDropTarget(null);
  };

  return (
    <div className="relative min-h-screen pb-24">
      {ambientBackdropEnabled && <AmbientBackground />}

      <div className="relative mx-auto flex max-w-7xl flex-col gap-10 px-6 pt-12">
        <BoardHeader
          projectName={snapshot.projectName}
          lastUpdated={snapshot.lastUpdated}
          onRename={setProjectName}
          onAddColumn={() => addColumn("New Column")}
          canAddColumn={canManageColumns}
          addColumnDisabledReason={addColumnDisabledReason}
          columnCount={snapshot.columns.length}
          totalGoalCount={totalGoalCount}
          completedGoalCount={completedGoalCount}
          showMetrics={boardMetricsEnabled}
          metrics={snapshot.metrics}
          capabilitySummary={state.capabilities.featureGates}
          capabilitiesLoading={state.capabilities.loading}
        />

        <div className="relative overflow-hidden rounded-[3rem] border border-white/10 bg-surface/60 p-8 shadow-[0_60px_160px_-60px_rgba(14,165,233,0.35)]">
          <div className="mt-4 flex gap-6 overflow-x-auto pb-4">
            {snapshot.columns.map((column) => (
              <BoardColumn
                key={column.id}
                column={column}
                onRemove={removeColumn}
                onRename={renameColumn}
                onAddGoal={(title, notes) => addGoal(column.id, title, notes ?? "")}
                onGoalOpen={(goal) => {
                  setActiveGoal(goal);
                  setActiveColumnId(column.id);
                }}
                enableComposer={quickComposerEnabled}
                onGoalDragStart={(goal, event) => handleGoalDragStart(column.id, goal, event)}
                onGoalDragOver={(goalId, event) => handleGoalDragOver(column.id, goalId, event)}
                onGoalDragLeave={(goalId) => handleGoalDragLeave(column.id, goalId)}
                onGoalDrop={(goalId, event) => handleGoalDrop(column.id, goalId, event)}
                onGoalDragEnd={handleGoalDragEnd}
                onColumnDragStart={(event) => handleColumnDragStart(column.id, event)}
                onColumnDragEnd={handleColumnDragEnd}
                onColumnSurfaceDragOver={(event) => handleColumnSurfaceDragOver(column.id, event)}
                onColumnSurfaceDragLeave={() => handleColumnSurfaceDragLeave(column.id)}
                onColumnSurfaceDrop={(event) => handleColumnSurfaceDrop(column.id, event)}
                isDraggingColumn={draggingColumnId === column.id}
                isColumnDropTarget={columnDropTarget === column.id}
                isGoalDropZoneActive={columnGoalDropTarget === column.id}
                draggingGoalId={draggingGoalId}
                dropTargetGoalId={
                  goalDropTarget && goalDropTarget.columnId === column.id ? goalDropTarget.goalId : null
                }
              />
            ))}
          </div>
        </div>
      </div>

      <CardEditor
        goal={activeGoal}
        columnId={activeColumnId}
        onClose={() => {
          setActiveGoal(null);
          setActiveColumnId(null);
        }}
        onUpdate={updateGoal}
        onDelete={removeGoal}
      />
    </div>
  );
}
