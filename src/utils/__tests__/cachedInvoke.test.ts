import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";

// Mock Tauri invoke
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { invoke as tauriInvoke } from "@tauri-apps/api/core";
const mockInvoke = vi.mocked(tauriInvoke);

import { cachedInvoke, refreshCached } from "@/composables/useCachedInvoke";

describe("cachedInvoke — déduplication et cache", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockInvoke.mockReset();
    vi.resetModules();
  });

  it("appelle invoke Tauri la première fois", async () => {
    mockInvoke.mockResolvedValue({ os: "Windows 11" });
    const result = await cachedInvoke<{ os: string }>("get_os");
    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(result.os).toBe("Windows 11");
  });

  it("retourne le cache au second appel sans re-invoquer Tauri", async () => {
    mockInvoke.mockResolvedValue({ os: "Windows 11" });
    await cachedInvoke("get_os");
    await cachedInvoke("get_os");
    expect(mockInvoke).toHaveBeenCalledTimes(1);
  });

  it("clés différentes → appels Tauri distincts", async () => {
    mockInvoke.mockResolvedValue([]);
    await cachedInvoke("get_disks");
    await cachedInvoke("get_temps");
    expect(mockInvoke).toHaveBeenCalledTimes(2);
  });

  it("args différents → clés de cache distinctes", async () => {
    mockInvoke.mockResolvedValue([]);
    await cachedInvoke("get_folder", { path: "C:\\" });
    await cachedInvoke("get_folder", { path: "D:\\" });
    expect(mockInvoke).toHaveBeenCalledTimes(2);
  });

  it("refreshCached() invalide le cache et re-fetche", async () => {
    mockInvoke.mockResolvedValue({ count: 1 });
    await cachedInvoke("get_apps");
    mockInvoke.mockResolvedValue({ count: 2 });
    const fresh = await refreshCached<{ count: number }>("get_apps");
    expect(fresh.count).toBe(2);
    expect(mockInvoke).toHaveBeenCalledTimes(2);
  });

  it("propage les erreurs Tauri sans les mettre en cache", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("WMI timeout"));
    await expect(cachedInvoke("get_wmi")).rejects.toThrow("WMI timeout");
    // Après erreur, un 2e appel doit re-tenter
    mockInvoke.mockResolvedValue({ ok: true });
    const result = await cachedInvoke<{ ok: boolean }>("get_wmi");
    expect(result.ok).toBe(true);
    expect(mockInvoke).toHaveBeenCalledTimes(2);
  });
});
