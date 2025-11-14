import { NextResponse } from "next/server";

import { assertUser } from "@/app/lib/session";
import type { VibeCard, WorkspaceBoard } from "@/app/components/board-types";
import { getBoard, getWorkspace } from "@/server/workspace-store";
import { planGoal, type GoalPayload } from "@/server/goal-planner";

function generateSuggestions(board: WorkspaceBoard) {
  const suggestions: { title: string; detail: string }[] = [];
  const totalGoals = board.columns.reduce((count, column) => count + column.goals.length, 0);
  const doneColumn = board.columns.find((column) => column.title.toLowerCase().includes("done"));
  const activeGoals = totalGoals - (doneColumn?.goals.length ?? 0);

  if (activeGoals > 8) {
    suggestions.push({
      title: "Focus the flow",
      detail: "More than eight active goals are in play. Spin up an automation to auto-archive stale work or split the board into swimlanes.",
    });
  }

  const hypeGoals = board.columns.flatMap((column) => column.goals.filter((goal) => goal.mood === "hype"));
  if (hypeGoals.length) {
    suggestions.push({
      title: "Capture the hype",
      detail: `There are ${hypeGoals.length} hype-tagged goals. Summon the Agent Factory to draft release notes while energy is high.`,
    });
  }

  const staleGoals = board.columns
    .flatMap((column) => column.goals)
    .filter((goal) => Date.now() - Date.parse(goal.createdAt) > 1000 * 60 * 60 * 24 * 7);
  if (staleGoals.length) {
    suggestions.push({
      title: "Refresh old vibes",
      detail: `${staleGoals.length} goals are older than a week. The retrieval engine can surface related context to reboot momentum.`,
    });
  }

  if (!suggestions.length) {
    suggestions.push({
      title: "Board is balanced",
      detail: "Momentum is healthy. Consider enabling automated release notes from the Agent Factory for bonus delight.",
    });
  }

  return suggestions;
}

function buildGoalPayload(
  body: unknown,
  context: { workspaceId: string; boardId: string; userId: string },
  board: WorkspaceBoard,
  focusCard: VibeCard | null,
  suggestions: { title: string; detail: string }[]
): GoalPayload {
  const baseContext = {
    boardSnapshot: board,
    focusCard,
    suggestions,
  };

  const payload: GoalPayload = {
    workspaceId: context.workspaceId,
    boardId: context.boardId,
    createdBy: context.userId,
    title: focusCard?.title ?? `${board.projectName} momentum`,
    summary: focusCard?.notes ?? board.description ?? "",
    focusCardId: focusCard?.id,
    context: baseContext,
  };

  if (body && typeof body === "object" && "goal" in body) {
    const rawGoal = (body as { goal?: Partial<GoalPayload> }).goal;
    if (rawGoal && typeof rawGoal === "object") {
      if (typeof rawGoal.id === "string" && rawGoal.id.trim()) {
        payload.id = rawGoal.id.trim();
      }
      if (typeof rawGoal.title === "string" && rawGoal.title.trim()) {
        payload.title = rawGoal.title.trim();
      }
      if (typeof rawGoal.summary === "string") {
        payload.summary = rawGoal.summary;
      }
      if (typeof rawGoal.focusCardId === "string") {
        payload.focusCardId = rawGoal.focusCardId;
      }
      if (rawGoal.context && typeof rawGoal.context === "object") {
        // Ensure critical baseContext fields are not overridden by user input
        payload.context = { ...rawGoal.context, ...baseContext };
      }
    }
  }

  return payload;
}

export async function POST(
  request: Request,
  { params }: { params: { workspaceId: string; boardId: string } }
) {
  const user = assertUser();
  const workspace = await getWorkspace(params.workspaceId);
  if (!workspace || !workspace.members.some((member) => member.id === user.id)) {
    return new NextResponse("Not Found", { status: 404 });
  }
  const board = await getBoard(params.workspaceId, params.boardId);
  if (!board) {
    return new NextResponse("Not Found", { status: 404 });
  }
  const suggestions = generateSuggestions(board);
  const focusCard = pickFocusCard(board);
  let body: unknown = undefined;
  try {
    body = await request.json();
  } catch {
    body = undefined;
  }

  const goalPayload = buildGoalPayload(
    body,
    {
      workspaceId: params.workspaceId,
      boardId: params.boardId,
      userId: user.id,
    },
    board,
    focusCard,
    suggestions
  );

  try {
    const plan = await planGoal(goalPayload, {
      uiApiBaseUrl: process.env.UI_API_URL ?? process.env.NEXT_PUBLIC_UI_API ?? undefined,
    });
    const goalId = goalPayload.id ?? plan.workflowId;
    const startedAt = new Date().toISOString();

    return NextResponse.json({
      suggestions,
      focusCard,
      plan: {
        goalId,
        goalTitle: goalPayload.title,
        workflowId: plan.workflowId,
        state: "pending",
        resumeToken: plan.resumeToken,
        startedAt,
        stages: plan.stages.map((stage) => ({
          id: stage.id,
          name: stage.name,
          state: "pending",
        })),
      },
    });
  } catch (error) {
    console.error("Failed to start workflow plan", error);
    return NextResponse.json(
      { suggestions, focusCard, error: "Failed to initialise planner" },
      { status: 502 }
    );
  }
}

function pickFocusGoal(board: WorkspaceBoard): Goal | null {
  const inProgress = board.columns.find((column) => column.title.toLowerCase().includes("progress"));
  if (!inProgress) {
    return board.columns[0]?.goals[0] ?? null;
  }
  const sorted = [...inProgress.goals].sort(
    (a, b) => Date.parse(a.createdAt) - Date.parse(b.createdAt)
  );
  return sorted[0] ?? null;
}
