import { PageSchema, WidgetSchema } from "@noa-ark/shared-ui/schema";
import { tokens } from "@noa-ark/shared-ui/tokens";
import { NotebookCitation, NotebookMediaAsset, NotebookSection, NotebookToPageOptions, ResearchNotebookRecord } from "./types";

function isoTimestamp(value?: string): string {
  if (!value) {
    return new Date().toISOString();
  }
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return new Date().toISOString();
  }
  return parsed.toISOString();
}

function resolveTags(tags?: string[]): string[] {
  if (!tags || tags.length === 0) {
    return ["research", "notebook"];
  }
  const unique = new Set(tags.map((tag) => tag.trim()).filter(Boolean));
  return Array.from(unique);
}

function collectCitations(
  notebook: ResearchNotebookRecord,
): NotebookCitation[] {
  const aggregated = new Map<string, NotebookCitation>();
  const push = (citation: NotebookCitation) => {
    if (!aggregated.has(citation.id)) {
      aggregated.set(citation.id, citation);
    }
  };

  notebook.metadata.citations?.forEach(push);
  notebook.sections.forEach((section) => section.citations?.forEach(push));
  notebook.appendix?.forEach((section) => section.citations?.forEach(push));

  return Array.from(aggregated.values());
}

function collectMedia(
  sections: NotebookSection[],
): NotebookMediaAsset[] {
  const bucket: NotebookMediaAsset[] = [];
  sections.forEach((section) => {
    section.media?.forEach((media) => bucket.push(media));
  });
  return bucket;
}

function makeWidget<TProps extends Record<string, unknown>>(
  id: string,
  kind: WidgetSchema["kind"],
  props: TProps,
): WidgetSchema {
  return {
    id,
    kind,
    props,
  };
}

interface NotebookGridOptions {
  layout: "stack" | "grid";
  columns?: string;
}

function gridForSurface(surface?: string): NotebookGridOptions {
  switch (surface) {
    case "desktop":
      return { layout: "grid", columns: "minmax(360px, 480px) minmax(360px, 1fr)" };
    case "mobile":
      return { layout: "stack" };
    default:
      return { layout: "grid", columns: "repeat(auto-fit, minmax(320px, 1fr))" };
  }
}

export function notebookToPageSchema(
  notebook: ResearchNotebookRecord,
  options: NotebookToPageOptions = {},
): PageSchema {
  const surface = options.surface ?? "web";
  const createdAt = isoTimestamp(notebook.metadata.createdAt);
  const updatedAt = isoTimestamp(notebook.metadata.updatedAt ?? notebook.metadata.createdAt);
  const tags = resolveTags(notebook.metadata.tags);
  const citations = collectCitations(notebook);
  const media = collectMedia([...notebook.sections, ...(notebook.appendix ?? [])]);

  const summaryWidget = makeWidget(`${notebook.metadata.id}-summary`, "research.notebook.summary", {
    title: notebook.metadata.title,
    summary: notebook.metadata.summary,
    author: notebook.metadata.author,
    reviewers: notebook.metadata.reviewers ?? [],
    createdAt,
    updatedAt,
    tags,
    shells: notebook.metadata.shells,
  });

  const sectionWidgets = notebook.sections.map((section, index) =>
    makeWidget(`${notebook.metadata.id}-section-${section.id || index}`, "research.notebook.section", {
      heading: section.heading,
      summary: section.summary,
      body: section.body,
      citations: section.citations ?? [],
      media: section.media ?? [],
      actions: section.actions ?? [],
      index,
    }),
  );

  const appendixWidgets = (notebook.appendix ?? []).map((section, index) =>
    makeWidget(`${notebook.metadata.id}-appendix-${section.id || index}`, "research.notebook.section", {
      heading: section.heading,
      summary: section.summary,
      body: section.body,
      citations: section.citations ?? [],
      media: section.media ?? [],
      actions: section.actions ?? [],
      index: notebook.sections.length + index,
      appendix: true,
    }),
  );

  const citationWidget = makeWidget(`${notebook.metadata.id}-citations`, "research.notebook.citations", {
    citations,
    caption: `Compiled ${citations.length} citation${citations.length === 1 ? "" : "s"}`,
  });

  const mediaWidget = makeWidget(`${notebook.metadata.id}-media`, "research.notebook.media", {
    assets: media,
    placeholderColor: tokens.colors["surface/overlay"],
  });

  const appendixRegion = appendixWidgets.length
    ? [
        {
          id: `${notebook.metadata.id}-appendix-region`,
          layout: "stack" as const,
          gap: tokens.spacing.lg,
          surface: "surface.secondary",
          widgets: appendixWidgets,
        },
      ]
    : [];

  const grid = gridForSurface(surface);

  return {
    id: notebook.metadata.id,
    version: "2024.12",
    kind: "workspace",
    metadata: {
      title: notebook.metadata.title,
      description: notebook.metadata.summary,
      tokensVersion: "2024.11",
      createdAt,
      updatedAt,
      accessibilityNotes: [
        "Research notebook sections are keyboard navigable with logical heading order.",
        "Media gallery surfaces alternative text and transcripts when provided.",
      ],
    },
    regions: [
      {
        id: `${notebook.metadata.id}-header`,
        layout: "stack",
        surface: "surface.glass",
        slot: "header",
        gap: tokens.spacing.lg,
        widgets: [summaryWidget],
      },
      {
        id: `${notebook.metadata.id}-sections`,
        layout: grid.layout,
        columns: grid.columns,
        surface: "surface.primary",
        gap: tokens.spacing.lg,
        widgets: sectionWidgets,
      },
      ...appendixRegion,
      {
        id: `${notebook.metadata.id}-resources`,
        layout: surface === "mobile" ? "stack" : "grid",
        columns: surface === "mobile" ? undefined : "minmax(280px, 1fr) minmax(280px, 1fr)",
        surface: "surface.secondary",
        gap: tokens.spacing.lg,
        widgets: [citationWidget, mediaWidget],
      },
    ],
  };
}
