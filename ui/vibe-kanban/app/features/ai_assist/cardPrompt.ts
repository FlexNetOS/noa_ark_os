"use client";
/**
 * Transforms Kanban cards into the canonical payload expected by the AI prompt endpoint.
 * Used by the AI button component inside the board UI.
 */

import type { VibeCard } from "../../components/board-types";

export type AIPromptPayload = {
  cardId: string;
  title: string;
  description?: string;
  labels?: string[];
  checklist?: string[];
  contextPaths?: string[];
};

export function buildPromptPayload(card: VibeCard): AIPromptPayload {
  const labels = card.integrations?.map((integration) => integration.label).filter(Boolean) ?? [];
  const checklist: string[] = [];

  if (card.dueDate) {
    checklist.push(`Verify due date ${new Date(card.dueDate).toLocaleDateString()}`);
  }
  if (card.integrations && card.integrations.length > 0) {
    checklist.push("Confirm integrations are synced and healthy");
  }

  const contextPaths = card.integrations?.map((integration) => `services/${integration.kind}`) ?? [];

  return {
    cardId: card.id,
    title: card.title,
    description: card.notes,
    labels: labels.length > 0 ? labels : undefined,
    checklist: checklist.length > 0 ? checklist : undefined,
    contextPaths: contextPaths.length > 0 ? contextPaths : undefined,
  };
}
