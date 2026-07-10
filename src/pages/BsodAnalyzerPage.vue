<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@/utils/invoke";
import { cachedInvoke, refreshCached } from "@/composables/useCachedInvoke";
import NCard from "@/components/ui/NCard.vue";
import NButton from "@/components/ui/NButton.vue";
import NBadge from "@/components/ui/NBadge.vue";
import NSpinner from "@/components/ui/NSpinner.vue";
import NSkeleton from "@/components/ui/NSkeleton.vue";
import { useNotificationStore } from "@/stores/notifications";
import { AlertTriangle, RefreshCw, FolderOpen, Clock, FileText, Info, ShieldAlert } from "lucide-vue-next";

const notify = useNotificationStore();

interface BsodEntry {
  timestamp: string; bug_check_code: string; bug_check_hex: string;
  description: string; parameters: string[]; module: string;
  dump_file: string; dump_size_kb: number;
}
interface BsodReport {
  entries: BsodEntry[]; total_count: number; last_bsod: string;
  dump_folder: string; dump_count: number;
}

const report = ref<BsodReport | null>(null);
const loading = ref(true);
const selected = ref<BsodEntry | null>(null);
const descriptions = ref<Record<string, string>>({});

async function loadReport(forceRefresh = false) {
  loading.value = true;
  try {
    report.value = forceRefresh
      ? await refreshCached<BsodReport>("get_bsod_history")
      : await cachedInvoke<BsodReport>("get_bsod_history");
  } catch (e: unknown) {
    notify.error("Erreur BSOD", String(e));
  }
  loading.value = false;
}

async function loadDescription(code: string) {
  if (descriptions.value[code]) return;
  try {
    const desc = await invoke<string>("get_bugcheck_description", { code });
    descriptions.value[code] = desc;
  } catch {
    descriptions.value[code] = "Description non disponible.";
  }
}

function selectEntry(e: BsodEntry) {
  selected.value = e;
  if (e.bug_check_code) loadDescription(e.bug_check_code);
}

async function openDumpFolder() {
  if (report.value?.dump_folder) {
    try { await invoke("open_path", { path: report.value.dump_folder }); } catch {}
  }
}

onMounted(loadReport);
</script>

<template>
  <div class="bsod-page">
    <div class="page-header">
      <div class="header-icon"><ShieldAlert :size="24" /></div>
      <div>
        <h1>Analyseur BSOD</h1>
        <p class="subtitle">Historique des écrans bleus (Blue Screen of Death)</p>
      </div>
      <NButton variant="ghost" size="sm" :loading="loading" @click="loadReport(true)" style="margin-left:auto">
        <RefreshCw :size="13" /> Actualiser
      </NButton>
    </div>

    <div v-if="loading" class="loading-wrap"><NSkeleton v-for="i in 4" :key="i" height="60px" style="margin-bottom:6px" /></div>

    <template v-else-if="report">
      <!-- Résumé -->
      <div class="stats-row">
        <div class="stat-card">
          <span class="stat-val" :class="report.total_count > 0 ? 'danger' : 'success'">
            {{ report.total_count }}
          </span>
          <span class="stat-label">BSOD total</span>
        </div>
        <div class="stat-card">
          <span class="stat-val">{{ report.dump_count }}</span>
          <span class="stat-label">Fichiers dump</span>
        </div>
        <div class="stat-card">
          <span class="stat-val mono">{{ report.last_bsod || '—' }}</span>
          <span class="stat-label">Dernier BSOD</span>
        </div>
        <div class="stat-card action" @click="openDumpFolder">
          <FolderOpen :size="20" />
          <span class="stat-label">Ouvrir dossier dump</span>
        </div>
      </div>

      <div v-if="report.total_count === 0" class="empty-state">
        <div class="success-icon">✓</div>
        <p>Aucun BSOD détecté — système stable</p>
      </div>

      <div v-else class="bsod-layout">
        <!-- Liste -->
        <NCard class="bsod-list-card">
          <template #header>
            <div class="section-header"><AlertTriangle :size="15" /><span>Événements ({{ report.entries.length }})</span></div>
          </template>
          <div class="bsod-list">
            <button
              v-for="e in report.entries" :key="e.timestamp"
              class="bsod-item"
              :class="{ active: selected?.timestamp === e.timestamp }"
              @click="selectEntry(e)"
            >
              <div class="bsod-item-left">
                <AlertTriangle :size="14" style="color:var(--danger);flex-shrink:0" />
                <div>
                  <span class="bsod-code">{{ e.bug_check_hex || e.bug_check_code }}</span>
                  <span class="bsod-ts"><Clock :size="10" /> {{ e.timestamp }}</span>
                </div>
              </div>
              <span class="bsod-module" v-if="e.module">{{ e.module }}</span>
            </button>
          </div>
        </NCard>

        <!-- Détail -->
        <NCard v-if="selected" class="bsod-detail-card">
          <template #header>
            <div class="section-header"><Info :size="15" /><span>Détail</span></div>
          </template>
          <div class="detail-body">
            <div class="detail-row"><span class="detail-key">Code</span><code class="detail-val">{{ selected.bug_check_hex }}</code></div>
            <div class="detail-row"><span class="detail-key">Horodatage</span><span class="detail-val">{{ selected.timestamp }}</span></div>
            <div class="detail-row" v-if="selected.module"><span class="detail-key">Module</span><code class="detail-val">{{ selected.module }}</code></div>
            <div class="detail-row" v-if="selected.dump_file"><span class="detail-key">Dump</span><code class="detail-val small">{{ selected.dump_file }}</code></div>
            <div class="detail-row" v-if="selected.dump_size_kb"><span class="detail-key">Taille</span><span class="detail-val">{{ Math.round(selected.dump_size_kb / 1024) }} Mo</span></div>

            <div class="desc-box" v-if="selected.description">
              <p class="desc-title">Description</p>
              <p class="desc-text">{{ selected.description }}</p>
            </div>

            <div class="desc-box" v-if="descriptions[selected.bug_check_code]">
              <p class="desc-title">Explication technique</p>
              <p class="desc-text">{{ descriptions[selected.bug_check_code] }}</p>
            </div>

            <div v-if="selected.parameters?.length" class="params-section">
              <p class="desc-title">Paramètres</p>
              <div class="params-list">
                <code v-for="(p, i) in selected.parameters" :key="i" class="param-item">{{ p }}</code>
              </div>
            </div>
          </div>
        </NCard>
        <NCard v-else class="bsod-detail-card empty-detail">
          <p style="color:var(--text-muted);font-size:13px">Cliquez sur un événement pour voir le détail.</p>
        </NCard>
      </div>
    </template>
  </div>
</template>

<style scoped>
.bsod-page { display: flex; flex-direction: column; gap: 14px; }
.page-header { display: flex; align-items: center; gap: 12px; }
.header-icon { width: 42px; height: 42px; border-radius: var(--radius-lg); background: var(--danger-muted); display: flex; align-items: center; justify-content: center; color: var(--danger); flex-shrink: 0; }
h1 { font-size: 22px; font-weight: 700; }
.subtitle { font-size: 12px; color: var(--text-muted); }
.loading-wrap { display: flex; flex-direction: column; gap: 6px; }
.stats-row { display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; }
@media (max-width: 700px) { .stats-row { grid-template-columns: 1fr 1fr; } }
.stat-card {
  padding: 14px 16px; background: var(--bg-secondary); border: 1px solid var(--border);
  border-radius: var(--radius-lg); display: flex; flex-direction: column; gap: 4px;
}
.stat-card.action { cursor: pointer; align-items: center; justify-content: center; }
.stat-card.action:hover { background: var(--bg-tertiary); }
.stat-val { font-size: 22px; font-weight: 800; color: var(--text-primary); }
.stat-val.danger { color: var(--danger); }
.stat-val.success { color: var(--success); }
.stat-val.mono { font-family: "JetBrains Mono", monospace; font-size: 14px; }
.stat-label { font-size: 11px; color: var(--text-muted); }
.empty-state { display: flex; flex-direction: column; align-items: center; gap: 10px; padding: 40px; }
.success-icon { width: 56px; height: 56px; border-radius: 50%; background: var(--success-muted); display: flex; align-items: center; justify-content: center; font-size: 24px; color: var(--success); }
.bsod-layout { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; align-items: start; }
@media (max-width: 900px) { .bsod-layout { grid-template-columns: 1fr; } }
.bsod-list-card, .bsod-detail-card { height: fit-content; }
.section-header { display: flex; align-items: center; gap: 8px; }
.bsod-list { display: flex; flex-direction: column; gap: 2px; }
.bsod-item {
  display: flex; align-items: center; justify-content: space-between; gap: 10px;
  padding: 10px 12px; border: none; border-radius: var(--radius-md);
  background: transparent; cursor: pointer; font-family: inherit; text-align: left;
  width: 100%; transition: background var(--transition-fast);
  border-left: 3px solid transparent;
}
.bsod-item:hover { background: var(--bg-tertiary); }
.bsod-item.active { background: var(--danger-muted); border-left-color: var(--danger); }
.bsod-item-left { display: flex; align-items: center; gap: 10px; }
.bsod-code { display: block; font-family: "JetBrains Mono", monospace; font-size: 12px; font-weight: 700; color: var(--text-primary); }
.bsod-ts { display: flex; align-items: center; gap: 4px; font-size: 10px; color: var(--text-muted); margin-top: 2px; }
.bsod-module { font-size: 10px; color: var(--text-muted); font-family: monospace; flex-shrink: 0; }
.detail-body { display: flex; flex-direction: column; gap: 8px; }
.detail-row { display: flex; gap: 12px; align-items: baseline; }
.detail-key { font-size: 11px; font-weight: 700; color: var(--text-muted); width: 80px; flex-shrink: 0; }
.detail-val { font-size: 13px; color: var(--text-primary); }
.detail-val.small { font-size: 10px; }
code { font-family: "JetBrains Mono", monospace; }
.desc-box { padding: 10px 12px; background: var(--bg-tertiary); border-radius: var(--radius-md); border-left: 3px solid var(--info); }
.desc-title { font-size: 11px; font-weight: 700; color: var(--info); text-transform: uppercase; letter-spacing: .06em; margin-bottom: 6px; }
.desc-text { font-size: 12px; color: var(--text-secondary); line-height: 1.6; }
.params-section { margin-top: 4px; }
.params-list { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 6px; }
.param-item { font-family: "JetBrains Mono", monospace; font-size: 11px; padding: 3px 8px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--text-secondary); }
.empty-detail { display: flex; align-items: center; justify-content: center; min-height: 120px; }
</style>
