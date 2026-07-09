import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useDataCache } from "@/stores/dataCache";

describe("dataCache — TTL et expiration", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("set/get retourne la valeur fraîche", () => {
    const cache = useDataCache();
    cache.set("get_system_info", { os: "Windows 11" });
    expect(cache.get<{ os: string }>("get_system_info")).toEqual({ os: "Windows 11" });
  });

  it("get retourne undefined pour clé inexistante", () => {
    const cache = useDataCache();
    expect(cache.get("unknown_cmd")).toBeUndefined();
  });

  it("has retourne true pour entrée fraîche", () => {
    const cache = useDataCache();
    cache.set("get_tools", []);
    expect(cache.has("get_tools")).toBe(true);
  });

  it("has retourne false pour clé absente", () => {
    const cache = useDataCache();
    expect(cache.has("missing")).toBe(false);
  });

  it("entrée expirée retourne undefined à la lecture", () => {
    const cache = useDataCache();
    cache.set("test_cmd", "data", 1000);
    vi.advanceTimersByTime(1001);
    expect(cache.get("test_cmd")).toBeUndefined();
  });

  it("get supprime l'entrée expirée du cache", () => {
    const cache = useDataCache();
    cache.set("test_cmd", "data", 500);
    vi.advanceTimersByTime(501);
    cache.get("test_cmd");
    expect(Object.keys(cache.cache).includes("test_cmd")).toBe(false);
  });

  it("invalidate supprime une entrée existante", () => {
    const cache = useDataCache();
    cache.set("get_gpu_detailed", { gpu: "RTX 4090" });
    cache.invalidate("get_gpu_detailed");
    expect(cache.has("get_gpu_detailed")).toBe(false);
  });

  it("clear vide tout le cache", () => {
    const cache = useDataCache();
    cache.set("cmd1", 1);
    cache.set("cmd2", 2);
    cache.clear();
    expect(Object.keys(cache.cache)).toHaveLength(0);
  });

  it("purgeExpired supprime seulement les entrées expirées", () => {
    const cache = useDataCache();
    cache.set("fresh", "ok", 10_000);
    cache.set("stale", "old", 100);
    vi.advanceTimersByTime(200);
    cache.purgeExpired();
    expect(cache.has("fresh")).toBe(true);
    expect(Object.keys(cache.cache).includes("stale")).toBe(false);
  });

  it("TTL de get_system_info est 5 minutes (300000ms)", () => {
    const cache = useDataCache();
    cache.set("get_system_info", { data: true });
    vi.advanceTimersByTime(299_000);
    expect(cache.has("get_system_info")).toBe(true);
    vi.advanceTimersByTime(2_000); // dépasse 5 min
    expect(cache.has("get_system_info")).toBe(false);
  });

  it("TTL de get_gpu_detailed est 30 minutes", () => {
    const cache = useDataCache();
    cache.set("get_gpu_detailed", { vendor: "NVIDIA" });
    vi.advanceTimersByTime(29 * 60_000);
    expect(cache.has("get_gpu_detailed")).toBe(true);
    vi.advanceTimersByTime(2 * 60_000);
    expect(cache.has("get_gpu_detailed")).toBe(false);
  });

  it("commande inconnue utilise le TTL par défaut (60s)", () => {
    const cache = useDataCache();
    cache.set("custom_unknown_cmd", { x: 1 });
    vi.advanceTimersByTime(59_000);
    expect(cache.has("custom_unknown_cmd")).toBe(true);
    vi.advanceTimersByTime(2_000);
    expect(cache.has("custom_unknown_cmd")).toBe(false);
  });

  it("TTL override prend le dessus sur TTL_MAP", () => {
    const cache = useDataCache();
    cache.set("get_system_info", { data: 1 }, 500); // override 5min → 500ms
    vi.advanceTimersByTime(600);
    expect(cache.has("get_system_info")).toBe(false);
  });
});
