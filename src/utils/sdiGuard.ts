let _fallbackTimer: ReturnType<typeof setTimeout> | null = null;

/** Mark SDI as active to prevent shutdown. Auto-clears after 60s as fallback. */
export function sdiAcquire(): void {
  if (_fallbackTimer) clearTimeout(_fallbackTimer);
  window.__nitrite_sdi_active = true;
  _fallbackTimer = setTimeout(() => {
    window.__nitrite_sdi_active = false;
    _fallbackTimer = null;
  }, 60_000);
}

/** Mark SDI as done and cancel the fallback timer. */
export function sdiRelease(): void {
  if (_fallbackTimer) { clearTimeout(_fallbackTimer); _fallbackTimer = null; }
  window.__nitrite_sdi_active = false;
}
