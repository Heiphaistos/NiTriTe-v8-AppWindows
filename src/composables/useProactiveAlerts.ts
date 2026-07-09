import { ref, onUnmounted } from 'vue';
import { invoke } from "@/utils/invoke";

interface AlertThresholds {
  cpuTempCritical: number;   // °C
  gpuTempCritical: number;   // °C
  diskUsageWarn: number;     // %
  diskUsageCritical: number; // %
  ramUsageWarn: number;      // %
}

const DEFAULT_THRESHOLDS: AlertThresholds = {
  cpuTempCritical: 90,
  gpuTempCritical: 85,
  diskUsageWarn: 85,
  diskUsageCritical: 95,
  ramUsageWarn: 90,
};

export const activeAlerts = ref<Array<{
  id: string;
  type: 'temp' | 'disk' | 'ram' | 'smart';
  severity: 'warning' | 'critical';
  message: string;
  timestamp: Date;
  dismissed: boolean;
}>>([]);

let monitorInterval: ReturnType<typeof setInterval> | null = null;
let instanceCount = 0;

function addAlert(id: string, type: 'temp' | 'disk' | 'ram' | 'smart', severity: 'warning' | 'critical', message: string) {
  const existing = activeAlerts.value.find(a => a.id === id && !a.dismissed);
  if (existing) return; // déjà présent
  activeAlerts.value.push({ id, type, severity, message, timestamp: new Date(), dismissed: false });
  // Notification système (si permission)
  if ('Notification' in window && Notification.permission === 'granted') {
    new Notification(`Nitrite — ${severity === 'critical' ? '🔴' : '🟡'} ${message}`);
  }
}

export function dismissAlert(id: string) {
  const alert = activeAlerts.value.find(a => a.id === id);
  if (alert) alert.dismissed = true;
}

export function dismissAll() {
  activeAlerts.value.forEach(a => { a.dismissed = true; });
}

export function useProactiveAlerts(thresholds: AlertThresholds = DEFAULT_THRESHOLDS) {
  async function checkOnce() {
    try {
      // Vérifier températures
      const temps = await invoke<Array<{ sensor_name: string; temp_celsius: number; source: string }>>('get_temperatures').catch(() => []);
      for (const t of temps) {
        if (t.temp_celsius <= 0) continue;
        if (t.sensor_name.toLowerCase().includes('cpu') || t.source.toLowerCase().includes('cpu')) {
          if (t.temp_celsius >= thresholds.cpuTempCritical) {
            addAlert(`cpu-temp-${t.sensor_name}`, 'temp', 'critical', `CPU ${t.sensor_name}: ${t.temp_celsius}°C — Surchauffe critique!`);
          }
        }
        if (t.sensor_name.toLowerCase().includes('gpu')) {
          if (t.temp_celsius >= thresholds.gpuTempCritical) {
            addAlert(`gpu-temp-${t.sensor_name}`, 'temp', 'critical', `GPU ${t.sensor_name}: ${t.temp_celsius}°C — Surchauffe!`);
          }
        }
      }
    } catch {}

    try {
      // Vérifier utilisation disques/RAM
      const sysInfo = await invoke<any>('get_system_info').catch(() => null);
      if (sysInfo) {
        if (sysInfo.ram?.usage_percent >= thresholds.ramUsageWarn) {
          const sev = sysInfo.ram.usage_percent >= 95 ? 'critical' : 'warning';
          addAlert('ram-usage', 'ram', sev, `RAM: ${sysInfo.ram.usage_percent.toFixed(0)}% utilisée`);
        }
        if (sysInfo.disks) {
          for (const d of sysInfo.disks) {
            for (const p of (d.partitions || [])) {
              const label = p.mount_point || d.model || 'Disque';
              if (p.usage_percent >= thresholds.diskUsageCritical) {
                addAlert(`disk-${label}`, 'disk', 'critical', `Disque ${label}: ${p.usage_percent.toFixed(0)}% plein!`);
              } else if (p.usage_percent >= thresholds.diskUsageWarn) {
                addAlert(`disk-${label}`, 'disk', 'warning', `Disque ${label}: ${p.usage_percent.toFixed(0)}% utilisé`);
              }
            }
          }
        }
      }
    } catch {}

    try {
      // Vérifier SMART
      const smart = await invoke<any[]>('get_smart_info').catch(() => []);
      for (const s of smart) {
        if (s.reallocated_sectors > 0) {
          addAlert(`smart-realloc-${s.name}`, 'smart', 'critical',
            `${s.name}: ${s.reallocated_sectors} secteur(s) réalloué(s) — Défaillance imminente!`);
        }
        const healthy = ['ok', 'good', 'passed', 'sain', 'healthy'];
        if (s.health_status && !healthy.some(h => s.health_status.toLowerCase().includes(h))) {
          addAlert(`smart-health-${s.name}`, 'smart', 'critical', `${s.name}: État SMART dégradé (${s.health_status})`);
        }
      }
    } catch {}
  }

  function start(intervalMs = 60000) {
    instanceCount++;
    if (instanceCount > 1) return; // timer déjà actif — on incrémente juste le compteur
    checkOnce();
    monitorInterval = setInterval(checkOnce, intervalMs);
    if ('Notification' in window && Notification.permission === 'default') {
      Notification.requestPermission();
    }
  }

  function stop() {
    instanceCount = Math.max(0, instanceCount - 1);
    if (instanceCount === 0 && monitorInterval) {
      clearInterval(monitorInterval);
      monitorInterval = null;
    }
  }

  // Auto-cleanup quand le composant appelant est détruit
  onUnmounted(() => stop());

  return { start, stop, activeAlerts, dismissAlert, dismissAll, checkOnce };
}
