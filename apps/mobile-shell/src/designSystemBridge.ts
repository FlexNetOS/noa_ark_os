import { buildReactNativeTree } from "@noa-ark/shared-ui/renderers/native";
import { vibeDashboardEnvelope } from "@noa-ark/shared-ui/samples";
import type { PageSchema } from "@noa-ark/shared-ui/schema";

export function renderMobileScene(schema: PageSchema = vibeDashboardEnvelope.schema) {
  const tree = buildReactNativeTree(schema);
  return {
    viewTree: tree.viewTree,
    accessibility: tree.accessibility,
    resumeToken: vibeDashboardEnvelope.resumeToken,
  };
}
