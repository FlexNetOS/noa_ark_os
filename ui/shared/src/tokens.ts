export type ColorToken =
  | "background/base"
  | "background/elevated"
  | "surface/primary"
  | "surface/glass"
  | "accent/primary"
  | "accent/secondary"
  | "accent/tertiary"
  | "text/strong"
  | "text/subtle"
  | "border/subtle"
  | "status/success"
  | "status/warning"
  | "status/danger";

export type TypographyToken =
  | "display/lg"
  | "display/md"
  | "body/base"
  | "body/compact"
  | "mono/compact";

export type MotionToken = "transition/snappy" | "transition/calm" | "duration/slow";

export interface ElevationToken {
  readonly blur: number;
  readonly spread: number;
  readonly y: number;
  readonly opacity: number;
}

export interface DesignTokens {
  readonly colors: Record<ColorToken, string>;
  readonly typography: Record<TypographyToken, { fontSize: string; lineHeight: string; fontWeight: number }>;
  readonly spacing: Record<"xxs" | "xs" | "sm" | "md" | "lg" | "xl" | "xxl", string>;
  readonly radii: Record<"xs" | "sm" | "md" | "lg" | "pill" | "full", string>;
  readonly shadows: Record<"level-0" | "level-1" | "level-2", ElevationToken>;
  readonly motion: Record<MotionToken, string>;
}

export const tokens: DesignTokens = {
  colors: {
    "background/base": "#040109",
    "background/elevated": "#0E0A19",
    "surface/primary": "#141126",
    "surface/glass": "rgba(31, 20, 63, 0.6)",
    "accent/primary": "#6366F1",
    "accent/secondary": "#A855F7",
    "accent/tertiary": "#22D3EE",
    "text/strong": "#F8FAFC",
    "text/subtle": "rgba(226, 232, 240, 0.64)",
    "border/subtle": "rgba(99, 102, 241, 0.24)",
    "status/success": "#34D399",
    "status/warning": "#FBBF24",
    "status/danger": "#F87171",
  },
  typography: {
    "display/lg": { fontSize: "2.5rem", lineHeight: "2.75rem", fontWeight: 600 },
    "display/md": { fontSize: "1.75rem", lineHeight: "2rem", fontWeight: 600 },
    "body/base": { fontSize: "1rem", lineHeight: "1.625rem", fontWeight: 400 },
    "body/compact": { fontSize: "0.875rem", lineHeight: "1.25rem", fontWeight: 400 },
    "mono/compact": { fontSize: "0.75rem", lineHeight: "1rem", fontWeight: 500 },
  },
  spacing: {
    xxs: "0.125rem",
    xs: "0.25rem",
    sm: "0.5rem",
    md: "0.75rem",
    lg: "1rem",
    xl: "1.5rem",
    xxl: "2.5rem",
  },
  radii: {
    xs: "0.375rem",
    sm: "0.5rem",
    md: "0.75rem",
    lg: "1.5rem",
    pill: "9999px",
    full: "50%",
  },
  shadows: {
    "level-0": { blur: 0, spread: 0, y: 0, opacity: 0 },
    "level-1": { blur: 28, spread: -8, y: 16, opacity: 0.28 },
    "level-2": { blur: 48, spread: -12, y: 32, opacity: 0.4 },
  },
  motion: {
    "transition/snappy": "all 160ms cubic-bezier(0.4, 0, 0.2, 1)",
    "transition/calm": "all 280ms cubic-bezier(0.22, 1, 0.36, 1)",
    "duration/slow": "640ms",
  },
};

export function cssShadow(token: ElevationToken): string {
  return `0 ${token.y}px ${token.blur}px ${token.spread}px rgba(15, 23, 42, ${token.opacity})`;
}
