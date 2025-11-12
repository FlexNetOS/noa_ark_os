import { PageEnvelope, RealTimeEvent, ResumeToken } from "../schema";

class EventEmitter<Events extends Record<string, unknown>> {
  private readonly listeners = new Map<keyof Events, Set<(value: unknown) => void>>();

  emit<EventName extends keyof Events>(event: EventName, value: Events[EventName]): void {
    const handlers = this.listeners.get(event);
    if (!handlers) return;
    for (const handler of handlers) {
      (handler as (value: Events[EventName]) => void)(value);
    }
  }

  on<EventName extends keyof Events>(event: EventName, handler: (value: Events[EventName]) => void): void {
    const handlers = this.listeners.get(event) ?? new Set();
    handlers.add(handler as (value: unknown) => void);
    this.listeners.set(event, handlers);
  }

  off<EventName extends keyof Events>(event: EventName, handler: (value: Events[EventName]) => void): void {
    const handlers = this.listeners.get(event);
    if (!handlers) return;
    handlers.delete(handler as (value: unknown) => void);
    if (!handlers.size) {
      this.listeners.delete(event);
    }
  }
}

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
  private readonly emitter = new EventEmitter<Events>();
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
    this.emitter.on(event, handler);
  }

  off<EventName extends keyof Events>(event: EventName, handler: (value: Events[EventName]) => void) {
    this.emitter.off(event, handler);
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
