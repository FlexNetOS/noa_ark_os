export type LogLevel = "debug" | "info" | "warn" | "error";

export interface StructuredLogOptions {
  component: string;
  event: string;
  message?: string;
  outcome?: string;
  traceId?: string | null;
  context?: Record<string, unknown>;
  error?: unknown;
  level?: LogLevel;
}

function generateTraceId(): string {
  if (typeof globalThis !== "undefined" && globalThis.crypto && typeof globalThis.crypto.randomUUID === "function") {
    return globalThis.crypto.randomUUID();
  }

  const random = Math.random().toString(16).slice(2, 10);
  return `${Date.now().toString(16)}-${random}`;
}

function normalizeError(error: unknown) {
  if (!error) {
    return undefined;
  }

  if (error instanceof Error) {
    return {
      name: error.name,
      message: error.message,
      stack: error.stack,
    };
  }

  if (typeof error === "object") {
    return error;
  }

  return { message: String(error) };
}

function write(level: LogLevel, payload: Record<string, unknown>) {
  const line = JSON.stringify({
    level,
    timestamp: new Date().toISOString(),
    ...payload,
  });

  if (level === "error") {
    console.error(line);
    return;
  }

  if (level === "warn") {
    console.warn(line);
    return;
  }

  if (level === "debug") {
    console.debug(line);
    return;
  }

  console.log(line);
}

export function logEvent(options: StructuredLogOptions): string {
  const { component, event, message, outcome = "unknown", context, error } = options;
  const level = options.level ?? "info";
  const traceId = options.traceId?.toString().trim() || generateTraceId();

  const payload = {
    trace_id: traceId,
    component,
    event,
    message: message ?? event,
    outcome,
    context: context ?? {},
    error: normalizeError(error),
  };

  write(level, payload);
  return traceId;
}

export function logInfo(options: Omit<StructuredLogOptions, "level">) {
  return logEvent({ ...options, level: "info" });
}

export function logWarn(options: Omit<StructuredLogOptions, "level">) {
  return logEvent({ ...options, level: "warn" });
}

export function logError(options: Omit<StructuredLogOptions, "level">) {
  return logEvent({ ...options, level: "error" });
}

export function ensureTraceId(candidate?: string | null): string {
  if (candidate && candidate.trim().length > 0) {
    return candidate.trim();
  }

  return generateTraceId();
}
