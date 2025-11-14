import type { DragEvent as ReactDragEvent } from "react";

export const DRAG_DATA_TYPE = "application/x-vibe-kanban";

type DragGoalPayload = {
  type: "goal";
  goalId: string;
  columnId: string;
};

type DragColumnPayload = {
  type: "column";
  columnId: string;
};

export type DragPayload = DragGoalPayload | DragColumnPayload;

export function setDragData(event: ReactDragEvent, payload: DragPayload) {
  if (!event.dataTransfer) return;
  event.dataTransfer.effectAllowed = "move";
  event.dataTransfer.setData(DRAG_DATA_TYPE, JSON.stringify(payload));
  // Provide a text fallback to make debugging easier in dev tools.
  event.dataTransfer.setData("text/plain", payload.type);
}

export function readDragData(event: ReactDragEvent): DragPayload | null {
  if (!event.dataTransfer) return null;
  const raw = event.dataTransfer.getData(DRAG_DATA_TYPE);
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw) as DragPayload;
    if (parsed.type === "goal" || parsed.type === "column") {
      return parsed;
    }
    return null;
  } catch {
    return null;
  }
}
