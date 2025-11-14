import { NextResponse } from "next/server";

import { assertUser } from "@/app/lib/session";
import { getBoard, getWorkspace } from "@/server/workspace-store";
import type { Goal, WorkspaceBoard } from "@/app/components/board-types";

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

export async function POST(
  _request: Request,
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
  const focusGoal = pickFocusGoal(board);
  return NextResponse.json({ suggestions, focusGoal });
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
