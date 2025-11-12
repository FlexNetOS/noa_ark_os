"use client";

import { useMemo } from "react";
import type { ReactNode } from "react";
import type { ComponentRenderProps, RenderContext } from "@noa-ark/shared-ui/renderers/web";
import { createWebRenderer } from "@noa-ark/shared-ui/renderers/web";
import { tokens, cssShadow } from "@noa-ark/shared-ui/tokens";
import type { PageSchema, ResumeToken } from "@noa-ark/shared-ui/schema";

import { WorkspaceSwitcher } from "./WorkspaceSwitcher";
import { IntegrationStatus } from "./IntegrationStatus";
import { BoardShell } from "./BoardShell";
import { PresenceBar } from "./PresenceBar";
import { AssistPanel } from "./AssistPanel";
import { AnalyticsPanel } from "./AnalyticsPanel";
import { ActivityTimeline } from "./ActivityTimeline";
import type { BoardState } from "./useBoardState";
import type { SessionState } from "./useSession";

export interface SchemaDrivenRendererProps {
  schema: PageSchema;
  context: RenderContext & {
    data: {
      boardState: BoardState;
      session: SessionState;
      resumeToken?: ResumeToken;
    };
  };
}

const widgetRegistry = {
  "workspace.header": function WorkspaceHeader({ context }: ComponentRenderProps) {
    const { session, resumeToken } = context.data as SchemaDrivenRendererProps["context"]["data"];
    const name = session.user?.name ?? "";
    return (
      <div
        style={{
          background: tokens.colors["surface/glass"],
          borderRadius: tokens.radii.lg,
          border: `1px solid ${tokens.colors["border/subtle"]}`,
          padding: tokens.spacing.xl,
          boxShadow: cssShadow(tokens.shadows["level-1"]),
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <div>
          <p style={{ textTransform: "uppercase", letterSpacing: "0.4em", color: tokens.colors["text/subtle"], fontSize: "0.7rem" }}>
            NOA ARK OS
          </p>
          <h1
            style={{
              marginTop: tokens.spacing.sm,
              fontSize: tokens.typography["display/md"].fontSize,
              lineHeight: tokens.typography["display/md"].lineHeight,
              fontWeight: tokens.typography["display/md"].fontWeight,
            }}
          >
            Vibe Kanban Control Hub
          </h1>
          {resumeToken && (
            <p style={{ color: tokens.colors["text/subtle"], marginTop: tokens.spacing.sm, fontSize: "0.85rem" }}>
              Resume workflow <strong>{resumeToken.workflowId}</strong> from checkpoint <code>{resumeToken.checkpoint}</code>
            </p>
          )}
        </div>
        {name && (
          <div style={{ display: "flex", alignItems: "center", gap: tokens.spacing.lg }}>
            <div style={{ textAlign: "right" }}>
              <p style={{ fontWeight: 600 }}>{name}</p>
              <p style={{ textTransform: "uppercase", letterSpacing: "0.2em", color: tokens.colors["text/subtle"], fontSize: "0.7rem" }}>
                Collaborator
              </p>
            </div>
            <div
              style={{
                width: "3rem",
                height: "3rem",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                borderRadius: tokens.radii.full,
                background: `linear-gradient(135deg, ${tokens.colors["accent/primary"]}, ${tokens.colors["accent/secondary"]})`,
                fontWeight: 600,
              }}
            >
              {name.slice(0, 2).toUpperCase()}
            </div>
          </div>
        )}
      </div>
    );
  },
  "workspace.session": ({ context }: ComponentRenderProps) => {
    const { boardState } = context.data as SchemaDrivenRendererProps["context"]["data"];
    return (
      <WidgetSurface>
        <WorkspaceSwitcher state={boardState} />
      </WidgetSurface>
    );
  },
  "workspace.integrations": ({ context }: ComponentRenderProps) => {
    const { boardState } = context.data as SchemaDrivenRendererProps["context"]["data"];
    return (
      <WidgetSurface>
        <IntegrationStatus integrations={boardState.integrations} />
      </WidgetSurface>
    );
  },
  "workspace.board": ({ context }: ComponentRenderProps) => {
    const { boardState } = context.data as SchemaDrivenRendererProps["context"]["data"];
    return (
      <WidgetSurface>
        <BoardShell state={boardState} />
      </WidgetSurface>
    );
  },
  "workspace.presence": ({ context }: ComponentRenderProps) => {
    const { boardState } = context.data as SchemaDrivenRendererProps["context"]["data"];
    return (
      <WidgetSurface>
        <PresenceBar presence={boardState.presence} members={boardState.workspace?.members ?? []} />
      </WidgetSurface>
    );
  },
  "workspace.assist": ({ context }: ComponentRenderProps) => {
    const { boardState } = context.data as SchemaDrivenRendererProps["context"]["data"];
    return (
      <WidgetSurface>
        <AssistPanel assist={boardState.assist} onRequest={boardState.requestAssist} />
      </WidgetSurface>
    );
  },
  "workspace.analytics": ({ context }: ComponentRenderProps) => {
    const { boardState } = context.data as SchemaDrivenRendererProps["context"]["data"];
    return (
      <WidgetSurface>
        <AnalyticsPanel board={boardState.snapshot} />
      </WidgetSurface>
    );
  },
  "workspace.activity": ({ context }: ComponentRenderProps) => {
    const { boardState } = context.data as SchemaDrivenRendererProps["context"]["data"];
    return (
      <WidgetSurface>
        <ActivityTimeline activity={boardState.activity} />
      </WidgetSurface>
    );
  },
};

const renderer = createWebRenderer({ registry: widgetRegistry });

export function SchemaDrivenRenderer({ schema, context }: SchemaDrivenRendererProps) {
  const tree = useMemo(() => renderer.renderPage(schema, context), [schema, context]);
  return tree;
}

function WidgetSurface({ children }: { children: ReactNode }) {
  return (
    <div
      style={{
        background: tokens.colors["surface/primary"],
        borderRadius: tokens.radii.lg,
        border: `1px solid ${tokens.colors["border/subtle"]}`,
        boxShadow: cssShadow(tokens.shadows["level-1"]),
      }}
    >
      {children}
    </div>
  );
}
