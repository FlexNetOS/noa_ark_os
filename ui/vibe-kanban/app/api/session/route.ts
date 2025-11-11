import { NextResponse } from "next/server";

import { clearUserSession, readUserFromCookies, setUserSession } from "../../lib/session";

export async function GET() {
  const user = readUserFromCookies();
  if (!user) {
    return NextResponse.json({ user: null }, { status: 200 });
  }
  return NextResponse.json({ user });
}

export async function POST(request: Request) {
  const payload = await request.json();
  const name = String(payload.name ?? "").trim();
  const id = String(payload.id ?? "").trim() || `user-${Date.now()}`;
  if (!name) {
    return NextResponse.json({ error: "Name is required" }, { status: 400 });
  }
  setUserSession({ id, name });
  return NextResponse.json({ ok: true, id, name });
}

export async function DELETE() {
  clearUserSession();
  return NextResponse.json({ ok: true });
}
