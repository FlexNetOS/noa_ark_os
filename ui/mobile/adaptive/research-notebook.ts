import { buildResearchNotebookNativeTree } from "@noa-ark/research-notebook/renderers/native";
import { ResearchNotebookRecord } from "@noa-ark/research-notebook";

export function renderResearchNotebookMobile(notebook: ResearchNotebookRecord) {
  return buildResearchNotebookNativeTree(notebook, { surface: "mobile" });
}
