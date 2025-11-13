import { NextResponse } from "next/server";

import { assertUser } from "@/app/lib/session";
import type { VibeCard, WorkspaceBoard } from "@/app/components/board-types";
import { aiDatabase } from "@/server/ai-database";
import { appendGoalTrace, getGoalMemoryInsights } from "@/server/memory-store";
import { getBoard, getWorkspace } from "@/server/workspace-store";

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
  const goalId = `board:${params.boardId}`;
  const generatedAt = new Date().toISOString();
  aiDatabase.recordGoalLifecycleEvent({
    goalId,
    workspaceId: workspace.id,
    eventType: "assist.requested",
    status: "received",
    summary: `Assist requested by ${user.name ?? user.id}`,
    payload: { boardId: board.id, workspaceId: workspace.id },
  });
  const suggestions = generateSuggestions(board);
  const focusCard = pickFocusCard(board);
  aiDatabase.recordGoalLifecycleEvent({
    goalId,
    workspaceId: workspace.id,
    eventType: "assist.generated",
    status: "success",
    summary: `Generated ${suggestions.length} suggestions`,
    payload: { boardId: board.id, suggestionCount: suggestions.length },
  });
  aiDatabase.upsertGoalEmbedding({
    goalId,
    workspaceId: workspace.id,
    embeddingModel: "kanban.board.stats.v1",
    embedding: buildBoardEmbedding(board),
  });
  await appendGoalTrace({
    id: `${goalId}-assist-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`,
    goalId,
    workspaceId: workspace.id,
    boardId: board.id,
    actorId: user.id,
    actorName: user.name,
    action: "assist.generated",
    summary: `Assist suggestions generated`,
    metadata: { suggestionCount: suggestions.length },
    createdAt: generatedAt,
  });
  const memory = await getGoalMemoryInsights(goalId, workspace.id);
  const lifecycle = aiDatabase.listGoalLifecycleEvents(goalId, 25);
  const artifacts = aiDatabase.listArtifacts(goalId, 25);
  const similarGoals = aiDatabase.searchSimilarGoals(goalId, "kanban.board.stats.v1", 5);
  return NextResponse.json({
    suggestions,
    focusCard,
    memory: {
      summary: memory.summary,
      traceCount: memory.traceCount,
      lastSeen: memory.lastSeen,
      traces: memory.traces,
      lifecycle,
      artifacts,
      similarGoals,
    },
  });
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

function buildBoardEmbedding(board: WorkspaceBoard): number[] {
  const totalCards = board.columns.reduce((count, column) => count + column.cards.length, 0);
  const hypeCards = board.columns.flatMap((column) => column.cards.filter((card) => card.mood === "hype")).length;
  const doneColumn = board.columns.find((column) => column.title.toLowerCase().includes("done"));
  const completed = doneColumn?.cards.length ?? 0;
  const focusCards = board.columns
    .flatMap((column) => column.cards)
    .filter((card) => card.mood === "focus").length;
  return [totalCards, hypeCards, completed, focusCards].map((value) => Number(value));
}
