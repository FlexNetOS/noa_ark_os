import test from "node:test";
import assert from "node:assert/strict";
import { TelemetryClient, UnifiedShell, navigationInventory, entryPointsForSurface } from "../src/shell";
import { LocalAuthProvider } from "../src/shell/auth";

const telemetry = new TelemetryClient(10);

function createShell() {
  return new UnifiedShell({ workflowEndpoint: "wss://example.test", telemetry, authProviders: [new LocalAuthProvider()] });
}

test("inventory exposes entry points for each surface", () => {
  const shell = createShell();
  shell.registerSurface("web");
  shell.registerSurface("desktop");
  shell.registerSurface("cli");

  for (const surface of navigationInventory.surfaces) {
    const entries = shell.entryPoints(surface);
    assert.ok(entries.length > 0, `expected entry points for ${surface}`);
    const inventoryEntries = entryPointsForSurface(surface);
    assert.deepEqual(
      entries.map((entry) => entry.id),
      inventoryEntries.map((entry) => entry.id),
      `entry points should match inventory for ${surface}`,
    );
  }
});

test("navigate records telemetry and enforces surface availability", () => {
  const shell = createShell();
  shell.registerSurface("web");
  const entry = shell.navigate("web", "dashboard");
  assert.equal(entry.id, "dashboard");
  const events = telemetry.drain();
  assert.equal(events.length, 1);
  assert.equal(events[0].entryPointId, "dashboard");
  assert.equal(events[0].surface, "web");
});

test("component bindings resolve per surface", () => {
  const shell = createShell();
  const binding = shell.componentFor("session.switcher", "cli");
  assert.ok(binding);
  assert.equal(binding?.implementation.includes("SessionSwitcher"), true);
});

test("sign in selects configured provider", async () => {
  const shell = createShell();
  await shell.signIn("local");
  assert.equal(shell.currentSessionProvider(), "local");
});
