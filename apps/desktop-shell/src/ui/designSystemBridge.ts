import { createWebRenderer } from "@noa-ark/shared-ui/renderers/web";
import type { ReactElement } from "react";
import type { RenderContext } from "@noa-ark/shared-ui/renderers/web";
import { vibeDashboardEnvelope } from "@noa-ark/shared-ui/samples";
import { SessionContinuityClient } from "@noa-ark/shared-ui/session";
import type { PageSchema, ResumeToken } from "@noa-ark/shared-ui/schema";

/**
 * Desktop shell bridge that mounts the shared design system using a headless renderer.
 */
export class DesktopDesignSystemBridge {
  private readonly renderer = createWebRenderer({ registry: {} });
  private resumeToken?: ResumeToken;
  private schema: PageSchema = vibeDashboardEnvelope.schema;
  private renderCallback?: (tree: ReactElement) => void;

  constructor(private readonly context: RenderContext) {}

  onRender(callback: (tree: ReactElement) => void) {
    this.renderCallback = callback;
  }

  async hydrateFromServer(baseUrl: string): Promise<void> {
    const response = await fetch(`${baseUrl}/ui/pages/${this.schema.id}`);
    if (response.ok) {
      const envelope = await response.json();
      this.schema = envelope.schema;
      this.resumeToken = envelope.resumeToken;
    }
  }

  render() {
    const tree = this.renderer.renderPage(this.schema, {
      ...this.context,
      data: { resumeToken: this.resumeToken },
    });
    this.renderCallback?.(tree as ReactElement);
    return tree;
  }

  connectSession(streamUrl: string): SessionContinuityClient {
    const client = new SessionContinuityClient({ workflowEndpoint: streamUrl });
    client.on("workflow:resume", (token) => {
      this.resumeToken = token;
      this.render();
    });
    client.connectWebSocket();
    return client;
  }
}
