import { ShellTelemetryEvent } from "./types";

type Sink = (event: ShellTelemetryEvent) => void;

export class TelemetryClient {
  private buffer: ShellTelemetryEvent[] = [];
  private sinks: Set<Sink> = new Set();
  constructor(private readonly bufferSize: number = 200) {}

  record(event: ShellTelemetryEvent): void {
    this.buffer.push(event);
    if (this.buffer.length > this.bufferSize) {
      this.buffer.shift();
    }
    for (const sink of this.sinks) {
      sink(event);
    }
  }

  drain(): ShellTelemetryEvent[] {
    const events = [...this.buffer];
    this.buffer = [];
    return events;
  }

  registerSink(sink: Sink): void {
    this.sinks.add(sink);
  }

  unregisterSink(sink: Sink): void {
    this.sinks.delete(sink);
  }
}
