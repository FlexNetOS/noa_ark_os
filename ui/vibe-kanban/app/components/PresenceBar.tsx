"use client";

import type { PresenceUser, WorkspaceMember } from "./board-types";

type PresenceBarProps = {
  presence: PresenceUser[];
  members: WorkspaceMember[];
};

function getMemberForPresence(presence: PresenceUser, members: WorkspaceMember[]) {
  return members.find((member) => member.id === presence.userId);
}

export function PresenceBar({ presence, members }: PresenceBarProps) {
  const active = presence.map((user) => ({
    ...user,
    member: getMemberForPresence(user, members),
  }));

  return (
    <div className="rounded-3xl border border-white/10 bg-surface/70 p-5 text-white/70">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-sm font-semibold uppercase tracking-[0.3em] text-white/50">
            Live crew
          </h3>
          <p className="mt-1 text-xs text-white/40">
            Presence pulses every few seconds so everyone feels connected.
          </p>
        </div>
        <span className="rounded-full border border-emerald-400/30 bg-emerald-500/10 px-3 py-1 text-xs font-medium text-emerald-200">
          {active.length} online
        </span>
      </div>
      <div className="mt-4 flex flex-wrap gap-3">
        {active.length === 0 && (
          <p className="text-sm text-white/40">You are opening the space. Ping the crew!</p>
        )}
        {active.map((item) => (
          <div
            key={item.userId}
            className="flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3 py-2"
          >
            <span
              className="inline-flex h-8 w-8 items-center justify-center rounded-full text-sm font-semibold text-white"
              style={{
                background: `conic-gradient(from 90deg, hsla(${item.member?.avatarHue ?? 220}, 100%, 60%, 1), hsla(${(item.member?.avatarHue ?? 220) + 45}, 90%, 55%, 1))`,
              }}
            >
              {(item.member?.name ?? item.userName).slice(0, 2).toUpperCase()}
            </span>
            <div className="leading-tight">
              <div className="text-sm font-medium text-white">
                {item.member?.name ?? item.userName}
              </div>
              <div className="text-[11px] uppercase tracking-[0.2em] text-white/40">
                {item.status}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
