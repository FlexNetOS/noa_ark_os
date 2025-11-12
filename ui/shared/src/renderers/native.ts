import { PageSchema } from "../schema";
import { tokens } from "../tokens";

export interface NativeRenderResult {
  viewTree: unknown;
  accessibility: string[];
}

export function buildReactNativeTree(schema: PageSchema): NativeRenderResult {
  const baseView = {
    type: "View",
    props: {
      style: {
        flex: 1,
        backgroundColor: tokens.colors["background/base"],
        paddingHorizontal: tokens.spacing.xl,
        paddingVertical: tokens.spacing.xxl,
      },
    },
    children: schema.regions.map((region) => ({
      type: "Surface",
      props: {
        variant: region.surface ?? "surface.primary",
        layout: region.layout,
      },
      children: region.widgets.map((widget) => ({
        type: "Widget",
        kind: widget.kind,
        props: widget.props ?? {},
      })),
    })),
  };

  const accessibility = schema.metadata.accessibilityNotes ?? [
    "All primary actions expose accessibilityRole=button",
    "Color contrasts follow WCAG AA using shared tokens.",
  ];

  return { viewTree: baseView, accessibility };
}
