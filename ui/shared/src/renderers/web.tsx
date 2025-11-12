import React, { ReactElement } from "react";
import { LayoutRegion, PageSchema, WidgetSchema } from "../schema";
import { tokens, cssShadow } from "../tokens";

export interface WebComponentRegistry {
  [widgetKind: string]: React.ComponentType<ComponentRenderProps>;
}

export interface ComponentRenderProps {
  widget: WidgetSchema;
  context: RenderContext;
}

export interface RenderContext {
  resumeWorkflow: (workflowId: string, stageId?: string) => void;
  triggerEvent: (bindingId: string) => void;
  data?: Record<string, unknown>;
}

export interface RenderOptions {
  registry: WebComponentRegistry;
}

export function createWebRenderer({ registry }: RenderOptions) {
  function renderWidget(widget: WidgetSchema, context: RenderContext): ReactElement {
    const Component = registry[widget.kind];
    if (Component) {
      return <Component key={widget.id} widget={widget} context={context} />;
    }

    return (
      <div
        key={widget.id}
        style={{
          border: `1px dashed ${tokens.colors["border/subtle"]}`,
          color: tokens.colors["text/subtle"],
          padding: tokens.spacing.lg,
          borderRadius: tokens.radii.md,
        }}
      >
        Unknown widget: {widget.kind}
      </div>
    );
  }

  function renderRegion(region: LayoutRegion, context: RenderContext): ReactElement {
    const surface = region.surface ?? "surface.primary";
    const surfaceTokens = {
      background: tokens.colors["surface/primary"],
      border: `1px solid ${tokens.colors["border/subtle"]}`,
      radius: tokens.radii.lg,
      padding: tokens.spacing.lg,
      shadow: cssShadow(tokens.shadows["level-1"]),
    };

    const regionStyle: React.CSSProperties = (() => {
      if (region.layout === "grid") {
        return {
          display: "grid",
          gridTemplateColumns: region.columns ?? "repeat(auto-fit, minmax(280px, 1fr))",
          gap: region.gap ?? tokens.spacing.lg,
        };
      }

      if (region.layout === "stack") {
        return {
          display: "flex",
          flexDirection: "column",
          gap: region.gap ?? tokens.spacing.lg,
        };
      }

      return { display: "block" };
    })();

    return (
      <section
        key={region.id}
        style={{
          background: surfaceTokens.background,
          border: surface === "surface.glass" ? `1px solid ${tokens.colors["border/subtle"]}` : surfaceTokens.border,
          borderRadius: surfaceTokens.radius,
          padding: surface === "surface.glass" ? tokens.spacing.xl : surfaceTokens.padding,
          boxShadow: surfaceTokens.shadow,
        }}
      >
        <div style={regionStyle}>{region.widgets.map((widget) => renderWidget(widget, context))}</div>
      </section>
    );
  }

  function renderPage(schema: PageSchema, context: RenderContext): ReactElement {
    const headerRegions = schema.regions.filter((region) => region.slot === "header");
    const footerRegions = schema.regions.filter((region) => region.slot === "footer");
    const mainRegions = schema.regions.filter((region) => !region.slot || region.slot === "main");

    return (
      <div
        style={{
          minHeight: "100vh",
          background: tokens.colors["background/base"],
          color: tokens.colors["text/strong"],
          padding: `${tokens.spacing.xl} ${tokens.spacing.xxl}`,
          fontFamily: "'Inter', sans-serif",
          display: "flex",
          flexDirection: "column",
          gap: tokens.spacing.xl,
        }}
      >
        {headerRegions.length > 0 && (
          <header style={{ display: "grid", gap: tokens.spacing.lg }}>
            {headerRegions.map((region) => renderRegion(region, context))}
          </header>
        )}
        <main
          style={{
            display: "grid",
            gap: tokens.spacing.xl,
            gridTemplateColumns: "minmax(280px, 320px) 1fr minmax(280px, 320px)",
          }}
        >
          {mainRegions.map((region) => renderRegion(region, context))}
        </main>
        {footerRegions.length > 0 && (
          <footer style={{ display: "grid", gap: tokens.spacing.lg }}>
            {footerRegions.map((region) => renderRegion(region, context))}
          </footer>
        )}
      </div>
    );
  }

  return { renderPage };
}
