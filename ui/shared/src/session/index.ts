import mitt, { Emitter } from "mitt";
import { PageEnvelope, RealTimeEvent, ResumeToken } from "../schema";

type Events = {
  "workflow:update": RealTimeEvent;
  "workflow:resume": ResumeToken;
  "connection:open": undefined;
  "connection:closed": undefined;
};

export interface SessionContinuityOptions {
  workflowEndpoint: string;
  grpcEndpoint?: string;
}

export class SessionContinuityClient {
  private ws?: WebSocket;
  private readonly emitter: Emitter<Events> = mitt<Events>();
  constructor(private readonly options: SessionContinuityOptions) {}

  connectWebSocket(): void {
    if (this.ws) return;
    this.ws = new WebSocket(this.options.workflowEndpoint);
    this.ws.onopen = () => this.emitter.emit("connection:open", undefined);
    this.ws.onclose = () => {
      this.emitter.emit("connection:closed", undefined);
      this.ws = undefined;
    };
    this.ws.onmessage = (event) => {
      try {
        const payload = JSON.parse(String(event.data)) as RealTimeEvent | ResumeToken;
        if ((payload as RealTimeEvent).eventType) {
          this.emitter.emit("workflow:update", payload as RealTimeEvent);
        } else {
          this.emitter.emit("workflow:resume", payload as ResumeToken);
        }
      } catch (error) {
        console.warn("Failed to parse session event", error);
      }
    };
  }

  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = undefined;
    }
  }

  requestResume(token: ResumeToken): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: "resume", token }));
    }
  }

  on<EventName extends keyof Events>(event: EventName, handler: (value: Events[EventName]) => void) {
    this.emitter.on(event, handler as any);
  }

  off<EventName extends keyof Events>(event: EventName, handler: (value: Events[EventName]) => void) {
    this.emitter.off(event, handler as any);
  }
}

export interface SchemaClient {
  fetchPage(pageId: string): Promise<PageEnvelope>;
}

export function createSchemaClient(baseUrl: string): SchemaClient {
  return {
    async fetchPage(pageId: string) {
      const response = await fetch(`${baseUrl}/ui/pages/${pageId}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch schema: ${response.status}`);
      }
      return (await response.json()) as PageEnvelope;
    },
  };
}
