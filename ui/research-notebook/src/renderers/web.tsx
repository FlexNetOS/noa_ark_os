import { ComponentRenderProps, WebComponentRegistry } from "@noa-ark/shared-ui/renderers/web";
import { CitationList, MediaGallery, SectionCard, SummaryCard } from "./common";

export function createResearchNotebookWebRegistry(): WebComponentRegistry {
  const registry: WebComponentRegistry = {
    "research.notebook.summary": (props: ComponentRenderProps) => SummaryCard(props),
    "research.notebook.section": (props: ComponentRenderProps) => SectionCard(props),
    "research.notebook.citations": (props: ComponentRenderProps) => CitationList(props),
    "research.notebook.media": (props: ComponentRenderProps) => MediaGallery(props),
  };

  return registry;
}
