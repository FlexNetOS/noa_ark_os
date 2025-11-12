/**
 * Simple in-memory token bucket rate limiter used by the AI prompt API.
 * Keeps per-identity state and is safe for a single Next.js server instance.
 */

export class RateLimiter {
  private readonly buckets = new Map<string, { tokens: number; updatedAt: number }>();

  constructor(private readonly capacity: number, private readonly refillIntervalMs: number) {}

  consume(identity: string) {
    const now = Date.now();
    const existing = this.buckets.get(identity) ?? { tokens: this.capacity, updatedAt: now };
    const elapsed = now - existing.updatedAt;
    const refill = Math.floor(elapsed / this.refillIntervalMs);

    let tokens = existing.tokens;
    if (refill > 0) {
      tokens = Math.min(this.capacity, tokens + refill);
    }

    if (tokens <= 0) {
      this.buckets.set(identity, { tokens, updatedAt: now });
      return false;
    }

    this.buckets.set(identity, { tokens: tokens - 1, updatedAt: now });
    return true;
  }
}

const globalAny = globalThis as typeof globalThis & { __aiRateLimiter?: RateLimiter };

if (!globalAny.__aiRateLimiter) {
  globalAny.__aiRateLimiter = new RateLimiter(5, 60_000);
}

export const aiRateLimiter = globalAny.__aiRateLimiter;
