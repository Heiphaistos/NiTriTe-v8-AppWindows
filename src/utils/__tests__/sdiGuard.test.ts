import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

// sdiGuard is a module-level singleton — reset between tests by re-importing
let sdiAcquire: () => void;
let sdiRelease: () => void;

beforeEach(async () => {
  vi.useFakeTimers();
  // Reset window flag
  window.__nitrite_sdi_active = false;
  // Re-import module to reset singleton timer state
  vi.resetModules();
  const mod = await import("@/utils/sdiGuard");
  sdiAcquire = mod.sdiAcquire;
  sdiRelease = mod.sdiRelease;
});

afterEach(() => {
  vi.useRealTimers();
});

describe("sdiGuard", () => {
  it("sdiAcquire sets window flag to true", () => {
    sdiAcquire();
    expect(window.__nitrite_sdi_active).toBe(true);
  });

  it("sdiRelease clears window flag", () => {
    sdiAcquire();
    sdiRelease();
    expect(window.__nitrite_sdi_active).toBe(false);
  });

  it("fallback timer clears flag after 60s", () => {
    sdiAcquire();
    expect(window.__nitrite_sdi_active).toBe(true);
    vi.advanceTimersByTime(60_000);
    expect(window.__nitrite_sdi_active).toBe(false);
  });

  it("sdiRelease cancels the fallback timer", () => {
    sdiAcquire();
    sdiRelease();
    vi.advanceTimersByTime(60_000); // timer should have been cancelled
    expect(window.__nitrite_sdi_active).toBe(false); // still false — correct
  });

  it("second sdiAcquire resets the fallback timer", () => {
    sdiAcquire();
    vi.advanceTimersByTime(30_000); // halfway through timer
    sdiAcquire(); // reset timer
    vi.advanceTimersByTime(30_000); // original timer would have fired — but it was reset
    expect(window.__nitrite_sdi_active).toBe(true); // still active
    vi.advanceTimersByTime(30_000); // now the reset timer fires
    expect(window.__nitrite_sdi_active).toBe(false);
  });
});
