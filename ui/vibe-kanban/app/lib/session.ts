import { cookies } from "next/headers";

export type SessionUser = {
  id: string;
  name: string;
};

const COOKIE_NAME = "vibe-user";

export function readUserFromCookies(): SessionUser | undefined {
  const raw = cookies().get(COOKIE_NAME)?.value;
  if (!raw) return undefined;
  try {
    return JSON.parse(raw) as SessionUser;
  } catch (error) {
    console.warn("Failed to parse session cookie", error);
    return undefined;
  }
}

export function assertUser(): SessionUser {
  const user = readUserFromCookies();
  if (!user) {
    throw new Response("Unauthorized", { status: 401 });
  }
  return user;
}

export function setUserSession(user: SessionUser) {
  cookies().set({
    name: COOKIE_NAME,
    value: JSON.stringify(user),
    httpOnly: false,
    sameSite: "lax",
    path: "/",
    maxAge: 60 * 60 * 24 * 30,
  });
}

export function clearUserSession() {
  cookies().delete(COOKIE_NAME);
}
