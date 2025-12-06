"use client";
/**
 * Transforms Kanban goals into the canonical payload expected by the AI prompt endpoint.
 * Used by the AI button component inside the board UI.
 */

import type { Goal } from "../../components/board-types";

export type AIPromptPayload = {
  goalId: string;
  title: string;
  description?: string;
  labels?: string[];
  checklist?: string[];
  contextPaths?: string[];
};

export function buildPromptPayload(goal: Goal): AIPromptPayload {
  const labels = goal.integrations?.map((integration) => integration.label).filter(Boolean) ?? [];
  const checklist: string[] = [];

  if (goal.dueDate) {
    checklist.push(`Verify due date ${new Date(goal.dueDate).toLocaleDateString()}`);
  }
  if (goal.integrations && goal.integrations.length > 0) {
    checklist.push("Confirm integrations are synced and healthy");
  }

  const contextPaths = goal.integrations?.map((integration) => `services/${integration.kind}`) ?? [];

  return {
    goalId: goal.id,
    title: goal.title,
    description: goal.notes,
    labels: labels.length > 0 ? labels : undefined,
    checklist: checklist.length > 0 ? checklist : undefined,
    contextPaths: contextPaths.length > 0 ? contextPaths : undefined,
  };
}
