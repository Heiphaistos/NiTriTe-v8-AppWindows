import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { useClipboard } from "@/composables/useClipboard";

describe("useClipboard", () => {
  let writeText: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    vi.useFakeTimers();
    writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      value: { writeText },
      configurable: true,
      writable: true,
    });
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it("copy retourne true et passe le texte au clipboard", async () => {
    const { copy, copied } = useClipboard();
    const result = await copy("test text");
    expect(result).toBe(true);
    expect(writeText).toHaveBeenCalledWith("test text");
  });

  it("copied devient true après un copy réussi", async () => {
    const { copy, copied } = useClipboard();
    await copy("hello");
    expect(copied.value).toBe(true);
  });

  it("copied repasse à false après 2 secondes", async () => {
    const { copy, copied } = useClipboard();
    await copy("hello");
    expect(copied.value).toBe(true);
    vi.advanceTimersByTime(2000);
    expect(copied.value).toBe(false);
  });

  it("copied reste true avant les 2 secondes", async () => {
    const { copy, copied } = useClipboard();
    await copy("hello");
    vi.advanceTimersByTime(1999);
    expect(copied.value).toBe(true);
  });

  it("double copy repart le timer depuis zéro", async () => {
    const { copy, copied } = useClipboard();
    await copy("first");
    vi.advanceTimersByTime(1500);
    await copy("second");
    vi.advanceTimersByTime(1500);
    expect(copied.value).toBe(true); // 1500ms après le 2e copy
    vi.advanceTimersByTime(600);
    expect(copied.value).toBe(false);
  });

  it("copy retourne false si le clipboard échoue", async () => {
    writeText.mockRejectedValueOnce(new Error("Permission denied"));
    const { copy, copied } = useClipboard();
    const result = await copy("text");
    expect(result).toBe(false);
    expect(copied.value).toBe(false);
  });
});
