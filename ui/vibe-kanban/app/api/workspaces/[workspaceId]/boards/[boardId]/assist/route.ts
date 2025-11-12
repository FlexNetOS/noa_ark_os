import { NextResponse } from "next/server";

import { assertUser } from "../../../../../../lib/session";
import { getBoard, getWorkspace } from "../../../../../../../server/workspace-store";
import type { VibeCard, WorkspaceBoard } from "../../../../../../components/board-types";
import { assertUser } from "@/app/lib/session";
import { getBoard, getWorkspace } from "@/server/workspace-store";
import type { VibeCard, WorkspaceBoard } from "@/app/components/board-types";

function generateSuggestions(board: WorkspaceBoard) {
  const suggestions: { title: string; detail: string }[] = [];
  const totalCards = board.columns.reduce((count, column) => count + column.cards.length, 0);
  const doneColumn = board.columns.find((column) => column.title.toLowerCase().includes("done"));
  const activeCards = totalCards - (doneColumn?.cards.length ?? 0);

  if (activeCards > 8) {
    suggestions.push({
      title: "Focus the flow",
      detail: "More than eight active cards are in play. Spin up an automation to auto-archive stale work or split the board into swimlanes.",
    });
  }

  const hypeCards = board.columns.flatMap((column) => column.cards.filter((card) => card.mood === "hype"));
  if (hypeCards.length) {
    suggestions.push({
      title: "Capture the hype",
      detail: `There are ${hypeCards.length} hype-tagged cards. Summon the Agent Factory to draft release notes while energy is high.`,
    });
  }

  const staleCards = board.columns
    .flatMap((column) => column.cards)
    .filter((card) => Date.now() - Date.parse(card.createdAt) > 1000 * 60 * 60 * 24 * 7);
  if (staleCards.length) {
    suggestions.push({
      title: "Refresh old vibes",
      detail: `${staleCards.length} cards are older than a week. The retrieval engine can surface related context to reboot momentum.`,
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
  const focusCard = pickFocusCard(board);
  return NextResponse.json({ suggestions, focusCard });
}

function pickFocusCard(board: WorkspaceBoard): VibeCard | null {
  const inProgress = board.columns.find((column) => column.title.toLowerCase().includes("progress"));
  if (!inProgress) {
    return board.columns[0]?.cards[0] ?? null;
  }
  const sorted = [...inProgress.cards].sort(
    (a, b) => Date.parse(a.createdAt) - Date.parse(b.createdAt)
  );
  return sorted[0] ?? null;
}
