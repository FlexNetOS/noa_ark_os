import { createWebRenderer, RenderContext } from "@noa-ark/shared-ui/renderers/web";
import { notebookToPageSchema, ResearchNotebookRecord } from "@noa-ark/research-notebook";
import { createResearchNotebookDesktopRegistry } from "@noa-ark/research-notebook/renderers/desktop";

const desktopRegistry = createResearchNotebookDesktopRegistry();
const desktopRenderer = createWebRenderer({ registry: desktopRegistry });

export function renderResearchNotebookDesktop(
  notebook: ResearchNotebookRecord,
  context: RenderContext,
) {
  const schema = notebookToPageSchema(notebook, { surface: "desktop" });
  return desktopRenderer.renderPage(schema, context);
}
