import { cookies } from "next/headers";

export type SessionUser = {
  id: string;
  name: string;
};

const COOKIE_NAME = "vibe-user";

export async function readUserFromCookies(): Promise<SessionUser | undefined> {
  const cookieStore = await cookies();
  const raw = cookieStore.get(COOKIE_NAME)?.value;
  if (!raw) return undefined;
  try {
    return JSON.parse(raw) as SessionUser;
  } catch (error) {
    console.warn("Failed to parse session cookie", error);
    return undefined;
  }
}

export async function assertUser(): Promise<SessionUser> {
  const user = await readUserFromCookies();
  if (!user) {
    throw new Response("Unauthorized", { status: 401 });
  }
  return user;
}

export async function setUserSession(user: SessionUser): Promise<void> {
  const cookieStore = await cookies();
  cookieStore.set({
    name: COOKIE_NAME,
    value: JSON.stringify(user),
    httpOnly: false,
    sameSite: "lax",
    path: "/",
    maxAge: 60 * 60 * 24 * 30,
  });
}

export async function clearUserSession(): Promise<void> {
  const cookieStore = await cookies();
  cookieStore.delete(COOKIE_NAME);
}
