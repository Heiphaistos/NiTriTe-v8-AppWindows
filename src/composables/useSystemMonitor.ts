import { ref, onUnmounted } from "vue";
import { invoke, isTauriContext } from "@/utils/invoke";
import type { MonitorData } from "@/types/system";

function generateFakeMonitorData(): MonitorData {
  const rand = (min: number, max: number) =>
    Math.round((Math.random() * (max - min) + min) * 10) / 10;

  return {
    cpu_percent: rand(10, 85),
    cpu_per_core: Array.from({ length: 8 }, () => rand(5, 95)),
    ram_percent: rand(30, 75),
    ram_used_gb: rand(4, 12),
    disk_percent: rand(40, 80),
    network_up_kbs: rand(0, 5000),
    network_down_kbs: rand(0, 15000),
    temperature_cpu: rand(35, 80),
    temperature_gpu: rand(30, 75),
    battery: null,
    top_processes: [
      { pid: 1234, name: "chrome.exe", cpu_percent: rand(2, 15), memory_mb: rand(100, 800) },
      { pid: 5678, name: "code.exe", cpu_percent: rand(1, 10), memory_mb: rand(200, 600) },
      { pid: 9012, name: "node.exe", cpu_percent: rand(0, 8), memory_mb: rand(50, 300) },
    ],
    alerts: [],
    timestamp: Date.now(),
  };
}

export function useSystemMonitor(intervalMs: number = 2000) {
  // Plancher : un intervalle nul/NaN ferait tourner setInterval en boucle rapide.
  const safeIntervalMs = Number.isFinite(intervalMs) && intervalMs >= 250 ? intervalMs : 2000;
  const monitorData = ref<MonitorData | null>(null);
  const isConnected = ref(false);
  let unlisten: (() => void) | null = null;
  let devInterval: ReturnType<typeof setInterval> | null = null;

  async function start() {
    if (isConnected.value) return;

    if (!isTauriContext()) {
      monitorData.value = generateFakeMonitorData();
      isConnected.value = true;
      devInterval = setInterval(() => {
        monitorData.value = generateFakeMonitorData();
      }, safeIntervalMs);
      return;
    }

    try {
      const { listen } = await import("@tauri-apps/api/event");

      unlisten = await listen<MonitorData>("system-monitor", (event) => {
        monitorData.value = event.payload;
      });

      await invoke("start_monitoring", { interval_ms: safeIntervalMs });
      isConnected.value = true;
    } catch {
      isConnected.value = false;
    }
  }

  async function stop() {
    if (devInterval) {
      clearInterval(devInterval);
      devInterval = null;
    }

    if (unlisten) {
      unlisten();
      unlisten = null;
    }

    if (isTauriContext() && isConnected.value) {
      await invoke("stop_monitoring").catch(() => {});
    }

    isConnected.value = false;
  }

  onUnmounted(stop);

  return { monitorData, isConnected, start, stop };
}
