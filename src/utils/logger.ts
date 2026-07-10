import { ref } from "vue";
import { invoke, isTauriContext } from "@/utils/invoke";

// ── Types ─────────────────────────────────────────────────────────────────────

export type LogLevel  = "DEBUG" | "INFO" | "WARN" | "ERROR" | "CRITICAL";
export type LogSource = "VUE" | "ROUTER" | "TAURI" | "UNCAUGHT" | "PERF" | "STORE" | "SYSTEM" | "UI";

export interface LogEntry {
  id:          string;
  timestamp:   string;   // ISO 8601
  level:       LogLevel;
  source:      LogSource;
  message:     string;
  details?:    string;   // stack trace ou JSON sérialisé
  duration_ms?: number;  // durée Tauri
  session_id:  string;
}

// ── Session ID unique par lancement ───────────────────────────────────────────

const SESSION_ID = `s-${Date.now().toString(36)}`;

// ── Buffer réactif (2 000 entrées max) ────────────────────────────────────────

export const logBuffer = ref<LogEntry[]>([]);
const MAX_BUFFER = 2000;

// ── Stats réactives ───────────────────────────────────────────────────────────

export const logStats = ref({ debug: 0, info: 0, warn: 0, error: 0, critical: 0 });

// ── Séquence ──────────────────────────────────────────────────────────────────

let _seq = 0;
function nextId(): string { return `${Date.now().toString(36)}-${(++_seq).toString(36)}`; }

// ── Envoi vers Rust (fire-and-forget) ────────────────────────────────────────

async function persist(entry: LogEntry): Promise<void> {
  if (!isTauriContext()) return;
  try {
    await invoke("log_entry", { entry });
  } catch { /* le logger ne doit jamais throw */ }
}

// ── Core push ─────────────────────────────────────────────────────────────────

function push(entry: LogEntry): void {
  // Stats
  const k = entry.level.toLowerCase() as keyof typeof logStats.value;
  if (k in logStats.value) logStats.value[k]++;

  // Buffer circulaire
  logBuffer.value.push(entry);
  if (logBuffer.value.length > MAX_BUFFER) {
    logBuffer.value.splice(0, logBuffer.value.length - MAX_BUFFER);
  }

  // Persistance Rust (async, non-bloquant)
  persist(entry);
}

// ── Sérialisation détails ─────────────────────────────────────────────────────

function serializeDetails(details: unknown): string | undefined {
  if (details === undefined || details === null) return undefined;
  if (typeof details === "string") return details.slice(0, 8000);
  try {
    return JSON.stringify(details, Object.getOwnPropertyNames(details), 2).slice(0, 8000);
  } catch {
    return String(details).slice(0, 8000);
  }
}

// ── API publique ──────────────────────────────────────────────────────────────

function log(level: LogLevel, source: LogSource, message: string, details?: unknown, duration_ms?: number): void {
  push({
    id:          nextId(),
    timestamp:   new Date().toISOString(),
    level,
    source,
    message:     String(message).slice(0, 3000),
    details:     serializeDetails(details),
    duration_ms,
    session_id:  SESSION_ID,
  });
}

export const logger = {
  debug:    (source: LogSource, message: string, details?: unknown) => log("DEBUG",    source, message, details),
  info:     (source: LogSource, message: string, details?: unknown) => log("INFO",     source, message, details),
  warn:     (source: LogSource, message: string, details?: unknown) => log("WARN",     source, message, details),
  error:    (source: LogSource, message: string, details?: unknown) => log("ERROR",    source, message, details),
  critical: (source: LogSource, message: string, details?: unknown) => log("CRITICAL", source, message, details),

  /** Log un appel Tauri avec durée. Succès = DEBUG, lent (>5s) = WARN, échec = ERROR */
  tauri(cmd: string, durationMs: number, err?: unknown): void {
    if (err !== undefined) {
      log("ERROR", "TAURI", `[${cmd}] échec (${durationMs}ms)`, err, durationMs);
    } else if (durationMs > 5000) {
      log("WARN",  "TAURI", `[${cmd}] lent (${durationMs}ms)`, undefined, durationMs);
    } else {
      log("DEBUG", "TAURI", `[${cmd}] ok (${durationMs}ms)`, undefined, durationMs);
    }
  },

  /** Log une erreur Vue capturée par onErrorCaptured */
  vue(info: string, err: unknown): void {
    const msg = err instanceof Error ? err.message : String(err);
    log("ERROR", "VUE", `[${info}] ${msg}`, err instanceof Error ? err.stack : err);
  },

  /** Log une erreur Vue Router */
  router(err: unknown): void {
    const msg = err instanceof Error ? err.message : String(err);
    log("ERROR", "ROUTER", msg, err instanceof Error ? err.stack : err);
  },

  getBuffer:  () => logBuffer.value,
  getStats:   () => logStats.value,
  getSession: () => SESSION_ID,

  clearBuffer(): void {
    logBuffer.value = [];
    logStats.value  = { debug: 0, info: 0, warn: 0, error: 0, critical: 0 };
  },
};

// ── Intercepteurs globaux (à appeler une seule fois au boot) ──────────────────

let _handlersInstalled = false;

export function setupGlobalErrorHandlers(): void {
  if (_handlersInstalled) return;
  _handlersInstalled = true;

  // Exceptions JS non catchées
  window.addEventListener("error", (e: ErrorEvent) => {
    logger.critical("UNCAUGHT", e.message || "Uncaught JS error", {
      filename: e.filename,
      lineno:   e.lineno,
      colno:    e.colno,
      stack:    e.error?.stack,
    });
  });

  // Promesses non catchées
  window.addEventListener("unhandledrejection", (e: PromiseRejectionEvent) => {
    const msg = e.reason instanceof Error ? e.reason.message : String(e.reason ?? "unknown");
    logger.error("UNCAUGHT", `Unhandled rejection: ${msg}`,
      e.reason instanceof Error ? e.reason.stack : e.reason);
  });

  // Intercept console.error → ERROR
  const origError = console.error.bind(console);
  console.error = (...args: unknown[]) => {
    origError(...args);
    const msg = args.map(a => (a instanceof Error ? a.message : String(a))).join(" ");
    // filtre les messages Vue internes redondants avec onErrorCaptured
    if (!msg.includes("[Vue warn]") && !msg.includes("[Nitrite][Vue error]")) {
      const errArg = args.find(a => a instanceof Error);
      log("ERROR", "UNCAUGHT", msg, errArg instanceof Error ? errArg.stack : undefined);
    }
  };

  // Intercept console.warn → WARN
  const origWarn = console.warn.bind(console);
  console.warn = (...args: unknown[]) => {
    origWarn(...args);
    const msg = args.map(a => String(a)).join(" ");
    if (!msg.includes("[Vue warn]")) {
      log("WARN", "UNCAUGHT", msg);
    }
  };

  logger.info("SYSTEM", `Logger initialisé — session ${SESSION_ID}`);
}
