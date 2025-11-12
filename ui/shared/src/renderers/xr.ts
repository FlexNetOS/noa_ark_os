import { PageSchema } from "../schema";
import { tokens } from "../tokens";

export interface SpatialNode {
  id: string;
  primitive: "panel" | "card" | "badge" | "text";
  position: [number, number, number];
  rotation: [number, number, number];
  scale: [number, number, number];
  props: Record<string, unknown>;
}

export interface SpatialScene {
  ambient: string;
  nodes: SpatialNode[];
}

export function projectToSpatialScene(schema: PageSchema): SpatialScene {
  const nodes: SpatialNode[] = [];
  schema.regions.forEach((region, regionIndex) => {
    const baseX = regionIndex * 2.5;
    region.widgets.forEach((widget, widgetIndex) => {
      nodes.push({
        id: `${widget.id}-panel`,
        primitive: "panel",
        position: [baseX, 1.5 - widgetIndex * 0.8, -2.5],
        rotation: [0, 0, 0],
        scale: [1.8, 1, 0.1],
        props: {
          surface: region.surface ?? "surface.primary",
          accent: tokens.colors["accent/primary"],
          widgetKind: widget.kind,
        },
      });
    });
  });

  return {
    ambient: "nebula",
    nodes,
  };
}
