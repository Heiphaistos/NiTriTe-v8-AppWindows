import { describe, it, expect, beforeEach } from "vitest";
import { logger, logBuffer, logStats } from "@/utils/logger";

describe("logger — buffer et stats", () => {
  beforeEach(() => {
    logger.clearBuffer();
  });

  it("debug() ajoute une entrée DEBUG", () => {
    logger.debug("SYSTEM", "test debug");
    expect(logBuffer.value).toHaveLength(1);
    expect(logBuffer.value[0].level).toBe("DEBUG");
    expect(logBuffer.value[0].source).toBe("SYSTEM");
    expect(logBuffer.value[0].message).toBe("test debug");
  });

  it("info() ajoute une entrée INFO", () => {
    logger.info("VUE", "composant monté");
    expect(logBuffer.value[0].level).toBe("INFO");
    expect(logBuffer.value[0].source).toBe("VUE");
  });

  it("warn() ajoute une entrée WARN", () => {
    logger.warn("TAURI", "lenteur détectée");
    expect(logBuffer.value[0].level).toBe("WARN");
  });

  it("error() ajoute une entrée ERROR", () => {
    logger.error("ROUTER", "route inconnue");
    expect(logBuffer.value[0].level).toBe("ERROR");
  });

  it("critical() ajoute une entrée CRITICAL", () => {
    logger.critical("UNCAUGHT", "crash fatal");
    expect(logBuffer.value[0].level).toBe("CRITICAL");
  });

  it("incrémente les stats correctement", () => {
    logger.debug("SYSTEM", "a");
    logger.info("SYSTEM", "b");
    logger.warn("SYSTEM", "c");
    logger.error("SYSTEM", "d");
    logger.critical("SYSTEM", "e");
    expect(logStats.value.debug).toBe(1);
    expect(logStats.value.info).toBe(1);
    expect(logStats.value.warn).toBe(1);
    expect(logStats.value.error).toBe(1);
    expect(logStats.value.critical).toBe(1);
  });

  it("clearBuffer() vide le buffer et remet les stats à 0", () => {
    logger.error("VUE", "erreur");
    logger.clearBuffer();
    expect(logBuffer.value).toHaveLength(0);
    expect(logStats.value.error).toBe(0);
  });

  it("chaque entrée a un id unique", () => {
    logger.info("SYSTEM", "a");
    logger.info("SYSTEM", "b");
    const ids = logBuffer.value.map(e => e.id);
    expect(new Set(ids).size).toBe(2);
  });

  it("chaque entrée a un timestamp ISO", () => {
    logger.info("SYSTEM", "ts test");
    const ts = logBuffer.value[0].timestamp;
    expect(() => new Date(ts)).not.toThrow();
    expect(new Date(ts).toISOString()).toBe(ts);
  });

  it("tronque les messages trop longs (>3000 chars)", () => {
    const longMsg = "x".repeat(4000);
    logger.info("SYSTEM", longMsg);
    expect(logBuffer.value[0].message.length).toBeLessThanOrEqual(3000);
  });

  it("tauri() log DEBUG si commande rapide", () => {
    logger.tauri("get_sys", 200);
    expect(logBuffer.value[0].level).toBe("DEBUG");
    expect(logBuffer.value[0].duration_ms).toBe(200);
  });

  it("tauri() log WARN si commande lente (>5s)", () => {
    logger.tauri("get_wmi", 6000);
    expect(logBuffer.value[0].level).toBe("WARN");
  });

  it("tauri() log ERROR si commande échoue", () => {
    logger.tauri("get_sys", 300, new Error("réseau mort"));
    expect(logBuffer.value[0].level).toBe("ERROR");
  });

  it("vue() log ERROR avec message de l'erreur", () => {
    logger.vue("setup hook", new Error("prop undefined"));
    expect(logBuffer.value[0].level).toBe("ERROR");
    expect(logBuffer.value[0].source).toBe("VUE");
    expect(logBuffer.value[0].message).toContain("prop undefined");
  });

  it("router() log ERROR de navigation", () => {
    logger.router(new Error("route 404"));
    expect(logBuffer.value[0].level).toBe("ERROR");
    expect(logBuffer.value[0].source).toBe("ROUTER");
  });

  it("limite le buffer à MAX_BUFFER (2000) entrées", () => {
    for (let i = 0; i < 2050; i++) logger.debug("SYSTEM", `msg ${i}`);
    expect(logBuffer.value.length).toBeLessThanOrEqual(2000);
  });
});
