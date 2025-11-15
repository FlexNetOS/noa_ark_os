export type WidgetKind =
  | "workspace.header"
  | "workspace.session"
  | "workspace.board"
  | "workspace.uploads"
  | "workspace.analytics"
  | "workspace.activity"
  | "workspace.assist"
  | "workspace.planner"
  | "workspace.integrations"
  | "workspace.presence"
  | "workspace.automation"
  | "workspace.agentFactory"
  | "layout.region"
  | "layout.section"
  | "cta.primary"
  | "form.signin"
  | "research.notebook.summary"
  | "research.notebook.section"
  | "research.notebook.citations"
  | "research.notebook.media";

export interface EventBinding {
  widgetId: string;
  event: "tap" | "press" | "navigate" | "submit" | "resume";
  workflow: {
    id: string;
    action: "start" | "resume" | "cancel";
  };
}

export interface WidgetSchema {
  id: string;
  kind: WidgetKind;
  variant?: string;
  label?: string;
  description?: string;
  component?: string;
  layout?: {
    span?: number;
    align?: "start" | "center" | "end";
    fill?: boolean;
  };
  props?: Record<string, unknown>;
  children?: WidgetSchema[];
  events?: EventBinding[];
}

export interface LayoutRegion {
  id: string;
  layout: "grid" | "stack" | "surface";
  columns?: string;
  gap?: string;
  surface?: string;
  slot?: "header" | "main" | "footer";
  widgets: WidgetSchema[];
}

export interface PageMetadata {
  title: string;
  description?: string;
  tokensVersion: string;
  createdAt: string;
  updatedAt: string;
  accessibilityNotes?: string[];
}

export interface PageSchema {
  id: string;
  version: string;
  kind: "workspace" | "settings" | "scene";
  metadata: PageMetadata;
  regions: LayoutRegion[];
}

export interface RealTimeEvent {
  eventType: "workflow/state" | "workflow/stage" | "presence/update" | "notification";
  workflowId: string;
  payload: Record<string, unknown>;
  timestamp: string;
}

export interface ResumeToken {
  workflowId: string;
  stageId?: string;
  checkpoint: string;
  issuedAt: string;
  expiresAt: string;
}

export interface PageEnvelope {
  schema: PageSchema;
  realtime: RealTimeEvent[];
  resumeToken?: ResumeToken;
}
