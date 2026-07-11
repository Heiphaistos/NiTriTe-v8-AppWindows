import { defineStore } from "pinia";
import { ref } from "vue";

export interface Toast {
  id: string;
  type: "success" | "error" | "warning" | "info";
  title: string;
  message?: string;
  duration?: number;
}

// Plafond : évite qu'une rafale de notifications (ex: boucle d'erreurs) empile
// des dizaines de toasts et sature l'écran. Les plus anciens sont retirés.
const MAX_TOASTS = 6;

export const useNotificationStore = defineStore("notifications", () => {
  const toasts = ref<Toast[]>([]);
  const timers = new Map<string, ReturnType<typeof setTimeout>>();

  function addToast(toast: Omit<Toast, "id">) {
    const id = crypto.randomUUID();
    const entry: Toast = { ...toast, id };
    toasts.value.push(entry);
    while (toasts.value.length > MAX_TOASTS) {
      removeToast(toasts.value[0].id);
    }

    const duration = toast.duration ?? 5000;
    const timer = setTimeout(() => removeToast(id), duration);
    timers.set(id, timer);
    return id;
  }

  function removeToast(id: string) {
    const timer = timers.get(id);
    if (timer !== undefined) { clearTimeout(timer); timers.delete(id); }
    toasts.value = toasts.value.filter((t) => t.id !== id);
  }

  function success(title: string, message?: string) {
    return addToast({ type: "success", title, message });
  }

  function error(title: string, message?: string) {
    return addToast({ type: "error", title, message });
  }

  function warning(title: string, message?: string) {
    return addToast({ type: "warning", title, message });
  }

  function info(title: string, message?: string) {
    return addToast({ type: "info", title, message });
  }

  return { toasts, addToast, removeToast, success, error, warning, info };
});
