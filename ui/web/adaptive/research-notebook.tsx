import { createWebRenderer, RenderContext } from "@noa-ark/shared-ui/renderers/web";
import { notebookToPageSchema, ResearchNotebookRecord } from "@noa-ark/research-notebook";
import { createResearchNotebookWebRegistry } from "@noa-ark/research-notebook/renderers/web";

const notebookRegistry = createResearchNotebookWebRegistry();
const notebookRenderer = createWebRenderer({ registry: notebookRegistry });

export function renderResearchNotebookWeb(
  notebook: ResearchNotebookRecord,
  context: RenderContext,
) {
  const schema = notebookToPageSchema(notebook, { surface: "web" });
  return notebookRenderer.renderPage(schema, context);
}
