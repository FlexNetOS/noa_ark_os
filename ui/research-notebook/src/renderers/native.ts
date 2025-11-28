import type { PageSchema } from "@noa-ark/shared-ui/schema";
import { buildReactNativeTree, NativeRenderResult } from "@noa-ark/shared-ui/renderers/native";
import { notebookToPageSchema } from "../notebook";
import { NotebookToPageOptions, ResearchNotebookRecord } from "../types";

export interface NativeNotebookOptions extends NotebookToPageOptions {
  includeAppendix?: boolean;
}

type NotebookPageRegion = PageSchema["regions"][number];

export function buildResearchNotebookNativeTree(
  notebook: ResearchNotebookRecord,
  options: NativeNotebookOptions = {},
): NativeRenderResult {
  const schema = notebookToPageSchema(notebook, { surface: options.surface ?? "mobile" });
  if (options.includeAppendix === false) {
    schema.regions = schema.regions.filter((region: NotebookPageRegion) => !region.id.endsWith("appendix-region"));
  }
  return buildReactNativeTree(schema);
}
