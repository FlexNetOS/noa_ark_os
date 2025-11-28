import assert from "node:assert/strict";
import test from "node:test";
import type { PageSchema, WidgetSchema } from "@noa-ark/shared-ui/schema";
import { notebookToPageSchema, ResearchNotebookRecord } from "../src";

type PageRegion = PageSchema["regions"][number];

test("notebookToPageSchema produces research widgets", () => {
  const notebook: ResearchNotebookRecord = {
    metadata: {
      id: "demo-notebook",
      title: "LLM Evaluation Sprint",
      summary: "Benchmarking local fine-tunes across latency profiles.",
      shells: ["web", "desktop", "mobile"],
      author: "Research Ops",
      tags: ["llm", "evaluation"],
      createdAt: "2024-12-01T12:00:00Z",
      citations: [
        { id: "c1", label: "Model Eval Paper", url: "https://example.com/paper" },
      ],
    },
    sections: [
      {
        id: "overview",
        heading: "Summary",
        summary: "Key takeaways",
        body: ["Latency improved by 23%", "Accuracy held within 1% margin."],
        citations: [
          { id: "c2", label: "Lab Notebook", url: "https://example.com/lab" },
        ],
      },
    ],
  };

  const schema = notebookToPageSchema(notebook);
  assert.equal(schema.regions[0].widgets[0].kind, "research.notebook.summary");
  const sectionRegion = schema.regions.find((region: PageRegion) => region.id === "demo-notebook-sections");
  assert(sectionRegion, "sections region should exist");
  assert(sectionRegion!.widgets.some((widget: WidgetSchema) => widget.kind === "research.notebook.section"));

  const resourcesRegion = schema.regions.find((region: PageRegion) => region.id === "demo-notebook-resources");
  assert(resourcesRegion, "resources region should exist");
  const resourceKinds = new Set(resourcesRegion!.widgets.map((widget: WidgetSchema) => widget.kind));
  assert(resourceKinds.has("research.notebook.citations"));
  assert(resourceKinds.has("research.notebook.media"));
});
