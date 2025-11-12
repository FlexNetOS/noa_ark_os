import { projectToSpatialScene } from "@noa-ark/shared-ui/renderers/xr";
import { vibeDashboardEnvelope } from "@noa-ark/shared-ui/samples";
import type { PageSchema } from "@noa-ark/shared-ui/schema";

export function buildSpatialProjection(schema: PageSchema = vibeDashboardEnvelope.schema) {
  return projectToSpatialScene(schema);
}
