import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useNotificationStore } from "@/stores/notifications";

describe("useNotificationStore — toasts", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it("addToast() ajoute un toast et retourne un id", () => {
    const store = useNotificationStore();
    const id = store.addToast({ type: "info", title: "Test" });
    expect(id).toBeTruthy();
    expect(store.toasts).toHaveLength(1);
    expect(store.toasts[0].title).toBe("Test");
  });

  it("success() ajoute un toast de type success", () => {
    const store = useNotificationStore();
    store.success("Opération réussie");
    expect(store.toasts[0].type).toBe("success");
  });

  it("error() ajoute un toast de type error", () => {
    const store = useNotificationStore();
    store.error("Erreur critique", "Détail de l'erreur");
    expect(store.toasts[0].type).toBe("error");
    expect(store.toasts[0].message).toBe("Détail de l'erreur");
  });

  it("warning() ajoute un toast de type warning", () => {
    const store = useNotificationStore();
    store.warning("Attention");
    expect(store.toasts[0].type).toBe("warning");
  });

  it("info() ajoute un toast de type info", () => {
    const store = useNotificationStore();
    store.info("Information");
    expect(store.toasts[0].type).toBe("info");
  });

  it("removeToast() supprime le toast par id", () => {
    const store = useNotificationStore();
    const id = store.addToast({ type: "info", title: "À supprimer" });
    store.removeToast(id);
    expect(store.toasts).toHaveLength(0);
  });

  it("le toast disparaît automatiquement après la durée", () => {
    const store = useNotificationStore();
    store.addToast({ type: "success", title: "Auto-remove", duration: 3000 });
    expect(store.toasts).toHaveLength(1);
    vi.advanceTimersByTime(3001);
    expect(store.toasts).toHaveLength(0);
  });

  it("durée par défaut = 5000ms", () => {
    const store = useNotificationStore();
    store.addToast({ type: "info", title: "Défaut" });
    vi.advanceTimersByTime(4999);
    expect(store.toasts).toHaveLength(1);
    vi.advanceTimersByTime(2);
    expect(store.toasts).toHaveLength(0);
  });

  it("removeToast() annule le timer automatique", () => {
    const store = useNotificationStore();
    const id = store.addToast({ type: "info", title: "Timer annulé", duration: 3000 });
    store.removeToast(id);
    // Le timer ne doit pas planter même si le toast est déjà supprimé
    vi.advanceTimersByTime(3001);
    expect(store.toasts).toHaveLength(0);
  });

  it("plusieurs toasts coexistent", () => {
    const store = useNotificationStore();
    store.success("A");
    store.error("B");
    store.info("C");
    expect(store.toasts).toHaveLength(3);
  });

  it("chaque toast a un id unique", () => {
    const store = useNotificationStore();
    store.info("A");
    store.info("B");
    const ids = store.toasts.map(t => t.id);
    expect(new Set(ids).size).toBe(2);
  });
});
