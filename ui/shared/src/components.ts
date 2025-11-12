import { tokens } from "./tokens";

export type ComponentRole =
  | "layout/surface"
  | "layout/grid"
  | "layout/stack"
  | "text/title"
  | "text/label"
  | "text/body"
  | "action/primary"
  | "action/subtle"
  | "status/pill"
  | "chart/summary";

export interface InteractionPattern {
  id: string;
  description: string;
  recommendedFor: ComponentRole[];
  states: Array<"default" | "hover" | "pressed" | "focus" | "disabled" | "active">;
}

export interface ComponentTokenMap {
  role: ComponentRole;
  background?: string;
  border?: string;
  text?: string;
  accent?: string;
  radius?: keyof typeof tokens.radii;
  padding?: keyof typeof tokens.spacing;
  elevation?: keyof typeof tokens.shadows;
}

export interface ComponentBlueprint {
  id: string;
  label: string;
  tokens: ComponentTokenMap;
  patterns: InteractionPattern[];
  description: string;
}

export const interactionPatterns: InteractionPattern[] = [
  {
    id: "pressable-solid",
    description: "Solid buttons for primary actions with micro-interactions.",
    recommendedFor: ["action/primary"],
    states: ["default", "hover", "pressed", "focus", "disabled"],
  },
  {
    id: "pressable-ghost",
    description: "Transparent buttons for low-emphasis interactions.",
    recommendedFor: ["action/subtle", "status/pill"],
    states: ["default", "hover", "pressed", "focus", "disabled"],
  },
  {
    id: "card",
    description: "Elevated surface that groups widgets and analytics modules.",
    recommendedFor: ["layout/surface", "chart/summary"],
    states: ["default", "hover", "active"],
  },
];

export const componentCatalog: ComponentBlueprint[] = [
  {
    id: "surface.glass",
    label: "Glass Surface",
    description: "Blurred surface used for overlays and XR projections.",
    tokens: {
      role: "layout/surface",
      background: tokens.colors["surface/glass"],
      border: tokens.colors["border/subtle"],
      radius: "lg",
      padding: "xl",
      elevation: "level-1",
    },
    patterns: [interactionPatterns[2]],
  },
  {
    id: "surface.primary",
    label: "Primary Surface",
    description: "Default surface for dashboards across web, desktop, and mobile.",
    tokens: {
      role: "layout/surface",
      background: tokens.colors["surface/primary"],
      border: tokens.colors["border/subtle"],
      radius: "lg",
      padding: "lg",
      elevation: "level-0",
    },
    patterns: [interactionPatterns[2]],
  },
  {
    id: "button.primary",
    label: "Primary Action Button",
    description: "Gradient-highlighted button for irreversible actions.",
    tokens: {
      role: "action/primary",
      background: `linear-gradient(135deg, ${tokens.colors["accent/primary"]}, ${tokens.colors["accent/secondary"]})`,
      text: tokens.colors["text/strong"],
      radius: "pill",
      padding: "md",
      elevation: "level-1",
    },
    patterns: [interactionPatterns[0]],
  },
  {
    id: "button.ghost",
    label: "Ghost Button",
    description: "Border-only button used for filters and XR gestures.",
    tokens: {
      role: "action/subtle",
      border: tokens.colors["border/subtle"],
      text: tokens.colors["text/subtle"],
      radius: "pill",
      padding: "md",
    },
    patterns: [interactionPatterns[1]],
  },
  {
    id: "text.title",
    label: "Title Text",
    description: "Heading style used for views and modals.",
    tokens: {
      role: "text/title",
      text: tokens.typography["display/md"].fontSize,
    },
    patterns: [],
  },
];

export function resolveComponentBlueprint(id: string): ComponentBlueprint | undefined {
  return componentCatalog.find((component) => component.id === id);
}
