<script setup lang="ts">
import { ref, onMounted } from "vue";
import { cachedInvoke } from "@/composables/useCachedInvoke";
import type { SystemHistory } from "@/types/diagnostic";
import NBadge from "@/components/ui/NBadge.vue";
import NSpinner from "@/components/ui/NSpinner.vue";
import DiagBanner from "@/components/ui/DiagBanner.vue";
import { History, Clock, AlertTriangle, Package, Power } from "lucide-vue-next";

const data = ref<SystemHistory | null>(null);
const loading = ref(true);
const error = ref("");

onMounted(async () => {
  try {
    data.value = await cachedInvoke<SystemHistory>("get_system_history");
  } catch (e: unknown) { error.value = (e instanceof Error ? e.message : String(e)) || "Erreur"; }
  finally { loading.value = false; }
});

function uptimeStr(h: number) {
  if (h >= 24) return `${(h / 24).toFixed(1)} jours`;
  return `${h.toFixed(1)} h`;
}
</script>

<template>
  <div class="diag-tab-content">
    <DiagBanner :icon="History" title="Historique Événements" desc="Journaux Windows et événements système récents" color="slate" />

    <div v-if="loading" class="diag-loading"><div class="diag-spinner"></div> Analyse historique système...</div>
    <div v-else-if="error" style="color:var(--error)">⚠ {{ error }}</div>
    <div v-else-if="data" style="display:flex;flex-direction:column;gap:14px">

      <!-- Timeline système -->
      <div class="diag-section">
        <p class="diag-section-label" style="margin:0 0 8px 0">
          <Clock :size="13" style="display:inline;margin-right:4px" />Chronologie Système
        </p>
        <div class="info-grid">
          <div class="info-row"><span>Installation Windows</span>
            <code>{{ data.windows_install_date || 'Inconnu' }}</code>
          </div>
          <div class="info-row"><span>Durée d'utilisation (depuis install)</span>
            <span>{{ data.total_uptime_days_since_install > 0 ? data.total_uptime_days_since_install + ' jours' : 'N/A' }}</span>
          </div>
          <div class="info-row"><span>Dernier démarrage</span>
            <code>{{ data.last_boot_time || 'Inconnu' }}</code>
          </div>
          <div class="info-row"><span>Uptime actuel</span>
            <NBadge :variant="data.current_uptime_hours > 168 ? 'warning' : 'success'">
              {{ uptimeStr(data.current_uptime_hours) }}
            </NBadge>
          </div>
          <div class="info-row"><span>Dernier utilisateur connecté</span>
            <code>{{ data.last_logon_user || 'Inconnu' }}</code>
          </div>
          <div class="info-row"><span>Arrêts (30 jours)</span>
            <span>{{ data.shutdown_count }}</span>
          </div>
          <div class="info-row"><span>Hibernations (30 jours)</span>
            <span>{{ data.hibernation_count }}</span>
          </div>
        </div>
      </div>

      <!-- Stabilité -->
      <div class="diag-section">
        <p class="diag-section-label" style="margin:0 0 8px 0">Stabilité (7-30 jours)</p>
        <div class="info-grid">
          <div class="info-row"><span>BSODs (30 jours)</span>
            <NBadge :variant="data.bsod_count_30d > 0 ? 'danger' : 'success'">{{ data.bsod_count_30d }}</NBadge>
          </div>
          <div class="info-row"><span>Événements Critiques (7 jours)</span>
            <NBadge :variant="data.critical_events_7d > 5 ? 'danger' : data.critical_events_7d > 0 ? 'warning' : 'success'">
              {{ data.critical_events_7d }}
            </NBadge>
          </div>
          <div class="info-row"><span>Erreurs (7 jours)</span>
            <NBadge :variant="data.error_events_7d > 50 ? 'danger' : data.error_events_7d > 20 ? 'warning' : 'neutral'">
              {{ data.error_events_7d }}
            </NBadge>
          </div>
          <div class="info-row"><span>Avertissements (7 jours)</span>
            <NBadge :variant="data.warning_events_7d > 100 ? 'warning' : 'neutral'">{{ data.warning_events_7d }}</NBadge>
          </div>
        </div>
        <div v-if="data.bsod_list.length" style="margin-top:10px">
          <p style="font-size:12px;color:var(--text-secondary);margin-bottom:6px">Dates des BSODs :</p>
          <div style="display:flex;gap:6px;flex-wrap:wrap">
            <code v-for="(b, i) in data.bsod_list" :key="i"
              style="font-size:11px;background:var(--bg-secondary);padding:2px 8px;border-radius:4px;color:var(--error)">{{ b }}</code>
          </div>
        </div>
      </div>

      <!-- Événements critiques récents -->
      <div v-if="data.recent_critical.length" class="diag-section">
        <p class="diag-section-label" style="margin:0 0 8px 0">
          <AlertTriangle :size="13" style="display:inline;margin-right:4px" />Événements récents Critique/Erreur ({{ data.recent_critical.length }})
        </p>
        <div v-for="(e, i) in data.recent_critical" :key="i"
          style="padding:6px 0;border-bottom:1px solid var(--border)">
          <div style="display:flex;align-items:center;gap:8px;margin-bottom:2px">
            <code class="muted" style="font-size:10px;min-width:100px">{{ e.time }}</code>
            <NBadge :variant="e.level === 'Critique' ? 'danger' : 'warning'" style="font-size:9px">{{ e.level }}</NBadge>
            <NBadge variant="neutral" style="font-size:9px">ID:{{ e.id }}</NBadge>
            <strong style="font-size:11px">{{ e.source }}</strong>
          </div>
          <div class="muted" style="font-size:11px;padding-left:110px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">{{ e.message }}</div>
        </div>
      </div>

      <!-- Logiciels installés récemment (30 jours) -->
      <div v-if="data.recent_installs.length" class="diag-section">
        <p class="diag-section-label" style="margin:0 0 8px 0">
          <Package :size="13" style="display:inline;margin-right:4px" />Logiciels installés (30 derniers jours — {{ data.recent_installs.length }})
        </p>
        <div style="overflow-x:auto">
          <table style="width:100%;border-collapse:collapse;font-size:12px">
            <thead>
              <tr style="background:var(--bg-secondary)">
                <th style="padding:6px 10px;text-align:left;color:var(--text-secondary);font-weight:500">Nom</th>
                <th style="padding:6px 10px;text-align:left;color:var(--text-secondary);font-weight:500">Version</th>
                <th style="padding:6px 10px;text-align:left;color:var(--text-secondary);font-weight:500">Éditeur</th>
                <th style="padding:6px 10px;text-align:left;color:var(--text-secondary);font-weight:500">Date</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(s, i) in data.recent_installs" :key="i"
                style="border-bottom:1px solid var(--border)">
                <td style="padding:6px 10px">{{ s.name }}</td>
                <td style="padding:6px 10px;color:var(--text-secondary)">{{ s.version || '—' }}</td>
                <td style="padding:6px 10px;color:var(--text-secondary)">{{ s.publisher || '—' }}</td>
                <td style="padding:6px 10px"><code style="font-size:11px">{{ s.install_date }}</code></td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Désinstallations récentes -->
      <div v-if="data.recent_uninstalls.length" class="diag-section">
        <p class="diag-section-label" style="margin:0 0 8px 0">
          <Power :size="13" style="display:inline;margin-right:4px" />Désinstallations récentes ({{ data.recent_uninstalls.length }})
        </p>
        <div v-for="(u, i) in data.recent_uninstalls" :key="i"
          style="padding:4px 0;font-size:12px;border-bottom:1px solid var(--border);color:var(--text-secondary)">
          {{ u }}
        </div>
      </div>
    </div>
  </div>
</template>
