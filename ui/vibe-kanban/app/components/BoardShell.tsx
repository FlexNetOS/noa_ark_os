"use client";

import { useMemo, useState } from "react";
import type { DragEvent as ReactDragEvent } from "react";

import type { VibeCard } from "./board-types";
import type { BoardState } from "./useBoardState";
import { BoardColumn } from "./BoardColumn";
import { CardEditor } from "./CardEditor";
import { AmbientBackground } from "./AmbientBackground";
import { BoardHeader } from "./BoardHeader";
import { isFeatureEnabled } from "./featureFlags";
import { readDragData, setDragData } from "./drag-utils";

type CardTarget = {
  columnId: string;
  cardId: string;
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
    addCard,
    updateCard,
    removeCard,
    moveCardWithinColumn,
    moveCardToColumn,
    moveColumn,
    setProjectName,
  } = state;

  const [activeCard, setActiveCard] = useState<VibeCard | null>(null);
  const [activeColumnId, setActiveColumnId] = useState<string | null>(null);
  const [draggingCardId, setDraggingCardId] = useState<string | null>(null);
  const [draggingColumnId, setDraggingColumnId] = useState<string | null>(null);
  const [cardDropTarget, setCardDropTarget] = useState<CardTarget | null>(null);
  const [columnDropTarget, setColumnDropTarget] = useState<string | null>(null);
  const [columnCardDropTarget, setColumnCardDropTarget] = useState<string | null>(null);

  const totalCards = useMemo(
    () => snapshot?.columns.reduce((acc, column) => acc + column.cards.length, 0) ?? 0,
    [snapshot?.columns]
    [snapshot]
  );
  const completedCount = useMemo(() => {
    return (snapshot?.columns ?? [])
      .filter((column) => /done|complete|finished/i.test(column.title))
      .reduce((count, column) => count + column.cards.length, 0);
      .reduce((count, column) => count + column.cards.length, 0) ?? 0;
  }, [snapshot?.columns]);
  }, [snapshot]);

  if (!snapshot) {
    return null;
  }

  const handleCardDragStart = (columnId: string, card: VibeCard, event: ReactDragEvent<HTMLButtonElement>) => {
    setDragData(event, { type: "card", columnId, cardId: card.id });
    setDraggingCardId(card.id);
  };

  const handleCardDragOver = (columnId: string, cardId: string, event: ReactDragEvent<HTMLButtonElement>) => {
    const payload = readDragData(event);
    if (!payload || payload.type !== "card") return;
    event.preventDefault();
    setCardDropTarget({ columnId, cardId });
  };

  const handleCardDragLeave = (columnId: string, cardId: string) => {
    setCardDropTarget((current) => {
      if (!current) return null;
      if (current.columnId === columnId && current.cardId === cardId) {
        return null;
      }
      return current;
    });
  };

  const handleCardDrop = (columnId: string, cardId: string, event: ReactDragEvent<HTMLButtonElement>) => {
    const payload = readDragData(event);
    if (!payload || payload.type !== "card") return;
    event.preventDefault();

    if (payload.columnId === columnId) {
      if (payload.cardId !== cardId) {
        moveCardWithinColumn(columnId, payload.cardId, cardId);
      }
    } else {
      moveCardToColumn(payload.columnId, columnId, payload.cardId, cardId);
    }

    setDraggingCardId(null);
    setCardDropTarget(null);
    setColumnCardDropTarget(null);
  };

  const handleColumnSurfaceDragOver = (columnId: string, event: ReactDragEvent<HTMLDivElement>) => {
    const payload = readDragData(event);
    if (!payload) return;

    if (payload.type === "card") {
      event.preventDefault();
      setColumnCardDropTarget(columnId);
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

    if (payload.type === "card") {
      if (payload.columnId !== columnId) {
        moveCardToColumn(payload.columnId, columnId, payload.cardId);
      }
      setDraggingCardId(null);
    }

    if (payload.type === "column") {
      if (payload.columnId !== columnId) {
        moveColumn(payload.columnId, columnId);
      }
      setDraggingColumnId(null);
    }

    setCardDropTarget(null);
    setColumnDropTarget(null);
    setColumnCardDropTarget(null);
  };

  const handleColumnSurfaceDragLeave = (columnId: string) => {
    setColumnDropTarget((current) => (current === columnId ? null : current));
    setColumnCardDropTarget((current) => (current === columnId ? null : current));
  };

  const handleCardDragEnd = () => {
    setDraggingCardId(null);
    setCardDropTarget(null);
    setColumnCardDropTarget(null);
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
      {isFeatureEnabled("ambientBackdrop") && <AmbientBackground />}

      <div className="relative mx-auto flex max-w-7xl flex-col gap-10 px-6 pt-12">
        <BoardHeader
          projectName={snapshot.projectName}
          lastUpdated={snapshot.lastUpdated}
          onRename={setProjectName}
          onAddColumn={() => addColumn("New Column")}
          columnCount={snapshot.columns.length}
          totalCardCount={totalCards}
          completedCount={completedCount}
          showMetrics={isFeatureEnabled("boardMetrics")}
          metrics={snapshot.metrics}
        />

        <div className="relative overflow-hidden rounded-[3rem] border border-white/10 bg-surface/60 p-8 shadow-[0_60px_160px_-60px_rgba(14,165,233,0.35)]">
          <div className="mt-4 flex gap-6 overflow-x-auto pb-4">
            {snapshot.columns.map((column) => (
              <BoardColumn
                key={column.id}
                column={column}
                onRemove={removeColumn}
                onRename={renameColumn}
                onAddCard={(title, notes) => addCard(column.id, title, notes ?? "")}
                onCardOpen={(card) => {
                  setActiveCard(card);
                  setActiveColumnId(column.id);
                }}
                enableComposer={isFeatureEnabled("quickComposer")}
                onCardDragStart={(card, event) => handleCardDragStart(column.id, card, event)}
                onCardDragOver={(cardId, event) => handleCardDragOver(column.id, cardId, event)}
                onCardDragLeave={(cardId) => handleCardDragLeave(column.id, cardId)}
                onCardDrop={(cardId, event) => handleCardDrop(column.id, cardId, event)}
                onCardDragEnd={handleCardDragEnd}
                onColumnDragStart={(event) => handleColumnDragStart(column.id, event)}
                onColumnDragEnd={handleColumnDragEnd}
                onColumnSurfaceDragOver={(event) => handleColumnSurfaceDragOver(column.id, event)}
                onColumnSurfaceDragLeave={() => handleColumnSurfaceDragLeave(column.id)}
                onColumnSurfaceDrop={(event) => handleColumnSurfaceDrop(column.id, event)}
                isDraggingColumn={draggingColumnId === column.id}
                isColumnDropTarget={columnDropTarget === column.id}
                isCardDropZoneActive={columnCardDropTarget === column.id}
                draggingCardId={draggingCardId}
                dropTargetCardId={
                  cardDropTarget && cardDropTarget.columnId === column.id ? cardDropTarget.cardId : null
                }
              />
            ))}
          </div>
        </div>
      </div>

      <CardEditor
        card={activeCard}
        columnId={activeColumnId}
        onClose={() => {
          setActiveCard(null);
          setActiveColumnId(null);
        }}
        onUpdate={updateCard}
        onDelete={removeCard}
      />
    </div>
  );
}
