import { PageEnvelope } from "../schema";

export const vibeDashboardEnvelope: PageEnvelope = {
  schema: {
    id: "vibe-kanban",
    version: "2024.05",
    kind: "workspace",
    metadata: {
      title: "Vibe Kanban Control Hub",
      description: "Unified workspace view rendered from the NOA UI schema.",
      tokensVersion: "0.1.0",
      createdAt: "2024-05-01T00:00:00.000Z",
      updatedAt: "2024-05-15T00:00:00.000Z",
      accessibilityNotes: [
        "All interactive widgets expose aria-labels on the web renderer.",
        "Mobile renderer maps gestures to accessible actions.",
      ],
    },
    regions: [
      {
        id: "header",
        slot: "header",
        layout: "surface",
        surface: "surface.glass",
        widgets: [
          { id: "header.primary", kind: "workspace.header", component: "WorkspaceHeader" }
        ],
      },
      {
        id: "left-rail",
        layout: "stack",
        gap: "1.5rem",
        surface: "surface.glass",
        widgets: [
          { id: "workspace.switcher", kind: "workspace.session", component: "WorkspaceSwitcher" },
          { id: "integrations", kind: "workspace.integrations", component: "IntegrationStatus" },
        ],
      },
      {
        id: "primary",
        layout: "stack",
        gap: "1.5rem",
        surface: "surface.primary",
        widgets: [
          { id: "board", kind: "workspace.board", component: "BoardShell", layout: { fill: true } },
        ],
      },
      {
        id: "right-rail",
        layout: "stack",
        gap: "1.5rem",
        surface: "surface.glass",
        widgets: [
          { id: "presence", kind: "workspace.presence", component: "PresenceBar" },
          { id: "planner", kind: "workspace.planner", component: "PlannerPanel" },
          { id: "assist", kind: "workspace.assist", component: "AssistPanel" },
          { id: "uploads", kind: "workspace.uploads", component: "UploadPanel" },
          { id: "analytics", kind: "workspace.analytics", component: "AnalyticsPanel" },
          { id: "activity", kind: "workspace.activity", component: "ActivityTimeline" },
          { id: "automation", kind: "workspace.automation", component: "AutomationPanel" },
          { id: "agent-factory", kind: "workspace.agentFactory", component: "AgentFactoryPanel" },
        ],
      },
    ],
  },
  realtime: [],
  resumeToken: {
    workflowId: "workspace-sync",
    stageId: "board-load",
    checkpoint: "kanban/snapshot/last",
    issuedAt: "2024-05-20T08:00:00.000Z",
    expiresAt: "2024-05-20T12:00:00.000Z",
  },
};
