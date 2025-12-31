import React from "react";
import { ComponentRenderProps, WebComponentRegistry } from "@noa-ark/shared-ui/renderers/web";
import { CitationList, MediaGallery, SectionCard, SummaryCard } from "./common";

function withDesktopChrome(renderer: (props: ComponentRenderProps) => React.ReactElement) {
  return (props: ComponentRenderProps) => (
    <div style={{ maxWidth: "1080px", margin: "0 auto", width: "100%" }}>{renderer(props)}</div>
  );
}

export function createResearchNotebookDesktopRegistry(): WebComponentRegistry {
  const registry: WebComponentRegistry = {
    "research.notebook.summary": withDesktopChrome(SummaryCard),
    "research.notebook.section": withDesktopChrome(SectionCard),
    "research.notebook.citations": withDesktopChrome(CitationList),
    "research.notebook.media": withDesktopChrome(MediaGallery),
  };

  return registry;
}
