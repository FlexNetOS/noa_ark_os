import React from "react";
import { WidgetSchema } from "@noa-ark/shared-ui/schema";
import { tokens } from "@noa-ark/shared-ui/tokens";
import { ComponentRenderProps } from "@noa-ark/shared-ui/renderers/web";

interface SummaryProps {
  title?: string;
  summary?: string;
  author?: string;
  reviewers?: string[];
  createdAt?: string;
  updatedAt?: string;
  tags?: string[];
  shells?: string[];
}

interface SectionProps {
  heading?: string;
  summary?: string;
  body?: string[];
  citations?: { id: string; label: string; url?: string; snippet?: string }[];
  media?: { id: string; kind: string; uri: string; description?: string; credit?: string }[];
  actions?: { label: string; workflowId: string; stageId?: string }[];
  appendix?: boolean;
  index?: number;
}

interface CitationListProps {
  citations?: { id: string; label: string; url?: string; publisher?: string; snippet?: string }[];
  caption?: string;
}

interface MediaListProps {
  assets?: { id: string; kind: string; uri: string; description?: string; credit?: string; thumbnailUri?: string }[];
  placeholderColor?: string;
}

function widgetProps<T>(widget: WidgetSchema): T {
  return (widget.props ?? {}) as T;
}

function formatDate(value?: string): string | undefined {
  if (!value) {
    return undefined;
  }
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return undefined;
  }
  return parsed.toLocaleString();
}

export function SummaryCard({ widget }: ComponentRenderProps) {
  const props = widgetProps<SummaryProps>(widget);
  const created = formatDate(props.createdAt);
  const updated = formatDate(props.updatedAt);

  return (
    <article
      style={{
        display: "grid",
        gap: tokens.spacing.md,
        padding: tokens.spacing.xl,
        borderRadius: tokens.radii.xl,
        background: tokens.colors["surface/primary"],
        boxShadow: tokens.shadows["level-1"],
      }}
    >
      <header>
        <p
          style={{
            textTransform: "uppercase",
            color: tokens.colors["text/subtle"],
            letterSpacing: "0.08em",
            fontSize: "0.75rem",
            marginBottom: tokens.spacing.xs,
          }}
        >
          Research Notebook
        </p>
        <h1
          style={{
            margin: 0,
            fontSize: "1.75rem",
            color: tokens.colors["text/strong"],
          }}
        >
          {props.title}
        </h1>
        {props.summary && (
          <p
            style={{
              margin: `${tokens.spacing.sm} 0 0`,
              color: tokens.colors["text/muted"],
              maxWidth: "64ch",
              lineHeight: 1.6,
            }}
          >
            {props.summary}
          </p>
        )}
      </header>
      <section
        style={{
          display: "flex",
          flexWrap: "wrap",
          gap: tokens.spacing.lg,
        }}
      >
        {props.author && (
          <div>
            <span style={{ display: "block", color: tokens.colors["text/subtle"], fontSize: "0.75rem" }}>Author</span>
            <span style={{ fontWeight: 600 }}>{props.author}</span>
          </div>
        )}
        {props.reviewers && props.reviewers.length > 0 && (
          <div>
            <span style={{ display: "block", color: tokens.colors["text/subtle"], fontSize: "0.75rem" }}>Reviewed By</span>
            <span>{props.reviewers.join(", ")}</span>
          </div>
        )}
        {created && (
          <div>
            <span style={{ display: "block", color: tokens.colors["text/subtle"], fontSize: "0.75rem" }}>Created</span>
            <time>{created}</time>
          </div>
        )}
        {updated && (
          <div>
            <span style={{ display: "block", color: tokens.colors["text/subtle"], fontSize: "0.75rem" }}>Updated</span>
            <time>{updated}</time>
          </div>
        )}
        {props.shells && props.shells.length > 0 && (
          <div>
            <span style={{ display: "block", color: tokens.colors["text/subtle"], fontSize: "0.75rem" }}>Surfaces</span>
            <span>{props.shells.join(", ")}</span>
          </div>
        )}
      </section>
      {props.tags && props.tags.length > 0 && (
        <footer style={{ display: "flex", flexWrap: "wrap", gap: tokens.spacing.sm }}>
          {props.tags.map((tag) => (
            <span
              key={tag}
              style={{
                padding: `${tokens.spacing.xs} ${tokens.spacing.sm}`,
                borderRadius: tokens.radii.lg,
                background: tokens.colors["surface/overlay"],
                color: tokens.colors["text/subtle"],
                fontSize: "0.75rem",
              }}
            >
              #{tag}
            </span>
          ))}
        </footer>
      )}
    </article>
  );
}

export function SectionCard({ widget, context }: ComponentRenderProps) {
  const props = widgetProps<SectionProps>(widget);
  const borderColor = props.appendix
    ? tokens.colors["border/strong"]
    : tokens.colors["border/subtle"];

  return (
    <article
      style={{
        display: "grid",
        gap: tokens.spacing.md,
        padding: tokens.spacing.lg,
        borderRadius: tokens.radii.lg,
        border: `1px solid ${borderColor}`,
        background: tokens.colors["background/base"],
        boxShadow: tokens.shadows["level-1"],
      }}
    >
      <header style={{ display: "grid", gap: tokens.spacing.xs }}>
        <h2 style={{ margin: 0, fontSize: "1.25rem" }}>{props.heading}</h2>
        {props.summary && (
          <p style={{ margin: 0, color: tokens.colors["text/muted"], lineHeight: 1.5 }}>{props.summary}</p>
        )}
      </header>
      <section style={{ display: "grid", gap: tokens.spacing.sm }}>
        {props.body?.map((paragraph, index) => (
          <p key={index} style={{ margin: 0, lineHeight: 1.7 }}>
            {paragraph}
          </p>
        ))}
      </section>
      {props.media && props.media.length > 0 && (
        <div
          style={{
            display: "grid",
            gap: tokens.spacing.sm,
          }}
        >
          {props.media.map((asset) => (
            <figure
              key={asset.id}
              style={{
                margin: 0,
                display: "grid",
                gap: tokens.spacing.xs,
              }}
            >
              <div
                style={{
                  width: "100%",
                  paddingBottom: "56.25%",
                  position: "relative",
                  background: tokens.colors["surface/overlay"],
                  borderRadius: tokens.radii.md,
                  overflow: "hidden",
                }}
              >
                <span
                  style={{
                    position: "absolute",
                    inset: 0,
                    display: "grid",
                    placeItems: "center",
                    color: tokens.colors["text/subtle"],
                    fontSize: "0.85rem",
                    letterSpacing: "0.04em",
                  }}
                >
                  {asset.kind.toUpperCase()}
                </span>
              </div>
              {(asset.description || asset.credit) && (
                <figcaption style={{ fontSize: "0.85rem", color: tokens.colors["text/subtle"] }}>
                  {asset.description}
                  {asset.credit && (
                    <span style={{ display: "block" }}>Credit: {asset.credit}</span>
                  )}
                </figcaption>
              )}
            </figure>
          ))}
        </div>
      )}
      {props.citations && props.citations.length > 0 && (
        <aside
          style={{
            borderTop: `1px solid ${tokens.colors["border/subtle"]}`,
            paddingTop: tokens.spacing.sm,
            display: "grid",
            gap: tokens.spacing.xs,
          }}
        >
          <strong style={{ fontSize: "0.85rem", color: tokens.colors["text/subtle"] }}>Citations</strong>
          {props.citations.map((citation) => (
            <a
              key={citation.id}
              href={citation.url}
              style={{
                color: tokens.colors["accent/primary"],
                textDecoration: "none",
              }}
            >
              {citation.label}
            </a>
          ))}
        </aside>
      )}
      {props.actions && props.actions.length > 0 && (
        <footer style={{ display: "flex", flexWrap: "wrap", gap: tokens.spacing.sm }}>
          {props.actions.map((action) => (
            <button
              key={action.workflowId}
              type="button"
              onClick={() => context.resumeWorkflow(action.workflowId, action.stageId)}
              style={{
                padding: `${tokens.spacing.xs} ${tokens.spacing.md}`,
                borderRadius: tokens.radii.md,
                border: "none",
                background: tokens.colors["accent/primary"],
                color: tokens.colors["text/inverted"],
                fontWeight: 600,
                cursor: "pointer",
              }}
            >
              {action.label}
            </button>
          ))}
        </footer>
      )}
    </article>
  );
}

export function CitationList({ widget }: ComponentRenderProps) {
  const props = widgetProps<CitationListProps>(widget);
  if (!props.citations || props.citations.length === 0) {
    return (
      <div
        style={{
          padding: tokens.spacing.lg,
          borderRadius: tokens.radii.lg,
          border: `1px dashed ${tokens.colors["border/subtle"]}`,
          color: tokens.colors["text/subtle"],
          background: tokens.colors["surface/overlay"],
        }}
      >
        No citations were captured for this notebook.
      </div>
    );
  }

  return (
    <section
      style={{
        display: "grid",
        gap: tokens.spacing.sm,
        padding: tokens.spacing.lg,
        borderRadius: tokens.radii.lg,
        background: tokens.colors["surface/primary"],
        boxShadow: tokens.shadows["level-1"],
      }}
    >
      <header>
        <strong style={{ fontSize: "1rem" }}>Source Registry</strong>
        {props.caption && (
          <p style={{ margin: 0, color: tokens.colors["text/subtle"], fontSize: "0.85rem" }}>{props.caption}</p>
        )}
      </header>
      <ol style={{ margin: 0, paddingLeft: "1.25rem", display: "grid", gap: tokens.spacing.sm }}>
        {props.citations.map((citation) => (
          <li key={citation.id} style={{ lineHeight: 1.6 }}>
            <a
              href={citation.url}
              style={{ color: tokens.colors["accent/primary"], textDecoration: "none", fontWeight: 600 }}
            >
              {citation.label}
            </a>
            {citation.publisher && (
              <span style={{ display: "block", color: tokens.colors["text/subtle"], fontSize: "0.85rem" }}>
                {citation.publisher}
              </span>
            )}
            {citation.snippet && (
              <q style={{ display: "block", color: tokens.colors["text/muted"], fontSize: "0.85rem" }}>{citation.snippet}</q>
            )}
          </li>
        ))}
      </ol>
    </section>
  );
}

export function MediaGallery({ widget }: ComponentRenderProps) {
  const props = widgetProps<MediaListProps>(widget);
  const assets = props.assets ?? [];

  return (
    <section
      style={{
        display: "grid",
        gap: tokens.spacing.sm,
        padding: tokens.spacing.lg,
        borderRadius: tokens.radii.lg,
        background: tokens.colors["surface/primary"],
        boxShadow: tokens.shadows["level-1"],
      }}
    >
      <header>
        <strong style={{ fontSize: "1rem" }}>Media Gallery</strong>
        <p style={{ margin: 0, color: tokens.colors["text/subtle"], fontSize: "0.85rem" }}>
          {assets.length === 0
            ? "No media assets were attached to this notebook."
            : `${assets.length} asset${assets.length === 1 ? "" : "s"} attached`}
        </p>
      </header>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))",
          gap: tokens.spacing.md,
        }}
      >
        {assets.map((asset) => (
          <figure
            key={asset.id}
            style={{
              margin: 0,
              display: "grid",
              gap: tokens.spacing.xs,
            }}
          >
            <div
              style={{
                position: "relative",
                paddingBottom: asset.kind === "audio" ? "40%" : "56.25%",
                borderRadius: tokens.radii.md,
                background: props.placeholderColor ?? tokens.colors["surface/overlay"],
                display: "grid",
                placeItems: "center",
                color: tokens.colors["text/subtle"],
                textTransform: "uppercase",
                letterSpacing: "0.05em",
              }}
            >
              {asset.kind}
            </div>
            {(asset.description || asset.credit) && (
              <figcaption style={{ fontSize: "0.85rem", color: tokens.colors["text/subtle"] }}>
                {asset.description}
                {asset.credit && <span style={{ display: "block" }}>Credit: {asset.credit}</span>}
              </figcaption>
            )}
          </figure>
        ))}
      </div>
    </section>
  );
}
