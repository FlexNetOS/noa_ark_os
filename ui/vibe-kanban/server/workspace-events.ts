import { EventEmitter } from "events";

import type {
  ActivityEvent,
  GoalAutomationState,
  NotificationEvent,
  WorkspacePresenceState,
} from "../app/components/board-types";

type WorkspaceEventPayload = {
  type: "board-updated" | "activity" | "presence" | "notification" | "automation";
  workspaceId: string;
  boardId?: string;
  data: unknown;
};

type WorkspaceEventListener = (payload: WorkspaceEventPayload) => void;

type PresenceRecord = {
  userId: string;
  userName: string;
  status: "online" | "idle";
  lastPing: number;
  boardId?: string;
};

class WorkspaceEventHub {
  private emitter = new EventEmitter();
  private presence = new Map<string, Map<string, PresenceRecord>>();

  addListener(listener: WorkspaceEventListener) {
    this.emitter.on("workspace-event", listener);
    return () => this.emitter.off("workspace-event", listener);
  }

  broadcast(event: WorkspaceEventPayload) {
    this.emitter.emit("workspace-event", event);
  }

  publishBoardUpdate(workspaceId: string, boardId: string) {
    this.broadcast({ type: "board-updated", workspaceId, boardId, data: { boardId } });
  }

  publishActivity(workspaceId: string, activity: ActivityEvent) {
    this.broadcast({ type: "activity", workspaceId, boardId: activity.boardId, data: activity });
  }

  publishNotification(workspaceId: string, notification: NotificationEvent) {
    this.broadcast({ type: "notification", workspaceId, data: notification });
  }

  publishAutomation(
    workspaceId: string,
    boardId: string,
    cardId: string,
    automation: GoalAutomationState,
  ) {
    this.broadcast({
      type: "automation",
      workspaceId,
      boardId,
      data: { boardId, cardId, automation },
    });
  }

  heartbeat(workspaceId: string, boardId: string | undefined, userId: string, userName: string) {
    const workspacePresence = this.ensureWorkspacePresence(workspaceId);
    const existing = workspacePresence.get(userId);
    const record: PresenceRecord = {
      userId,
      userName,
      boardId,
      status: "online",
      lastPing: Date.now(),
    };
    workspacePresence.set(userId, { ...existing, ...record });
    this.broadcastPresence(workspaceId);
  }

  removePresence(workspaceId: string, userId: string) {
    const workspacePresence = this.ensureWorkspacePresence(workspaceId);
    workspacePresence.delete(userId);
    this.broadcastPresence(workspaceId);
  }

  snapshotPresence(workspaceId: string): WorkspacePresenceState {
    return this.getPresenceState(workspaceId);
  }

  sweepPresence() {
    const threshold = Date.now() - 30_000;
    for (const [workspaceId, users] of this.presence.entries()) {
      let changed = false;
      for (const [userId, record] of users.entries()) {
        if (record.lastPing < threshold) {
          users.delete(userId);
          changed = true;
        }
      }
      if (changed) {
        this.broadcastPresence(workspaceId);
      }
    }
  }

  private broadcastPresence(workspaceId: string) {
    const state = this.getPresenceState(workspaceId);
    this.broadcast({ type: "presence", workspaceId, data: state });
  }

  private getPresenceState(workspaceId: string): WorkspacePresenceState {
    const workspacePresence = this.ensureWorkspacePresence(workspaceId);
    const users = Array.from(workspacePresence.values()).map((record) => ({
      userId: record.userId,
      userName: record.userName,
      status: record.status,
      lastPing: new Date(record.lastPing).toISOString(),
    }));
    return { workspaceId, boardId: undefined, users };
  }

  private ensureWorkspacePresence(workspaceId: string) {
    if (!this.presence.has(workspaceId)) {
      this.presence.set(workspaceId, new Map());
    }
    return this.presence.get(workspaceId)!;
  }
}

const globalAny = globalThis as typeof globalThis & { __workspaceEventHub?: WorkspaceEventHub };

if (!globalAny.__workspaceEventHub) {
  globalAny.__workspaceEventHub = new WorkspaceEventHub();
  setInterval(() => globalAny.__workspaceEventHub?.sweepPresence(), 15_000).unref();
}

export const workspaceEventHub = globalAny.__workspaceEventHub;
