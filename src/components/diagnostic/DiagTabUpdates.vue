<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke, invokeRaw } from "@/utils/invoke";
import { listen } from "@tauri-apps/api/event";
import { useNotificationStore } from "@/stores/notifications";
import { useExportData } from '@/composables/useExportData';

const notify = useNotificationStore();
const { exportCSV } = useExportData();
import { RefreshCw, Download, CheckCircle, AlertTriangle, Package, Zap } from "lucide-vue-next";
import NBadge from "@/components/ui/NBadge.vue";
import NButton from "@/components/ui/NButton.vue";
import NSpinner from "@/components/ui/NSpinner.vue";
import NCollapse from "@/components/ui/NCollapse.vue";

interface InstalledUpdate { title: string; hotfix_id: string; description: string; installed_on: string; installed_by: string; }
interface PendingUpdate   { title: string; kb_ids: string; severity: string; size_mb: number; is_downloaded: boolean; }
interface WingetPackage   { name: string; id: string; version: string; available: string; source: string; }
interface ChocoPackage    { name: string; current_version: string; available_version: string; pinned: boolean; }
interface ScoopPackage    { name: string; installed: string; available: string; }

const props = defineProps<{ updatesHistory: InstalledUpdate[] }>();

// === État local ===
const scanningWU    = ref(false);
const pendingWU     = ref<PendingUpdate[]>([]);
const wuScanned     = ref(false);
const triggeringWU  = ref(false);
const wuMessage     = ref("");

const wingetList    = ref<WingetPackage[]>([]);
const wingetLoading = ref(false);
const wingetUpgrading = ref(false);
const wingetMsg     = ref("");
const wingetOk      = ref(false);

const chocoList     = ref<ChocoPackage[]>([]);
const chocoLoading  = ref(false);
const chocoInstalled = ref(false);
const chocoUpgrading = ref(false);
const chocoMsg      = ref("");

const scoopList     = ref<ScoopPackage[]>([]);
const scoopLoading  = ref(false);
const scoopInstalled = ref(false);
const scoopUpgrading = ref(false);
const scoopMsg      = ref("");
const scoopUpgradeLog = ref("");
let scoopUnlisten: (() => void) | null = null;

// === Windows Update ===
async function scanWU() {
  scanningWU.value = true; wuScanned.value = false; wuMessage.value = "";
  try { pendingWU.value = await invokeRaw<PendingUpdate[]>("scan_pending_windows_updates"); }
  catch { pendingWU.value = []; }
  wuScanned.value = true; scanningWU.value = false;
}
async function triggerWU() {
  triggeringWU.value = true;
  try { wuMessage.value = await invokeRaw<string>("trigger_windows_update"); }
  catch { wuMessage.value = "Erreur lors du déclenchement"; }
  triggeringWU.value = false;
}

// === Winget ===
async function loadWinget() {
  wingetLoading.value = true; wingetMsg.value = "";
  try {
    wingetOk.value = await invoke<boolean>("check_winget");
    if (wingetOk.value) wingetList.value = await invoke<WingetPackage[]>("list_upgradable");
  } catch { wingetOk.value = false; }
  wingetLoading.value = false;
}
async function upgradeWinget() {
  wingetUpgrading.value = true; wingetMsg.value = "Mise à jour en cours...";
  try {
    await invokeRaw("upgrade_all");
    wingetMsg.value = "Mise à jour winget terminée ✓";
    await loadWinget();
  } catch (e: any) { wingetMsg.value = "Erreur : " + e; }
  wingetUpgrading.value = false;
}

// === Chocolatey ===
async function loadChoco() {
  chocoLoading.value = true; chocoMsg.value = "";
  try {
    chocoInstalled.value = await invoke<boolean>("check_chocolatey");
    if (chocoInstalled.value) chocoList.value = await invoke<ChocoPackage[]>("list_chocolatey_upgrades");
  } catch { chocoInstalled.value = false; }
  chocoLoading.value = false;
}
async function upgradeChoco() {
  chocoUpgrading.value = true; chocoMsg.value = "Mise à jour en cours...";
  try {
    const r = await invokeRaw<{ success: boolean; upgraded_count: number; message: string }>("upgrade_chocolatey_all");
    chocoMsg.value = r?.success ? `${r.upgraded_count} paquet(s) mis à jour ✓` : `Erreur : ${r?.message || "inconnue"}`;
    await loadChoco();
  } catch (e: any) { chocoMsg.value = "Erreur : " + e; }
  chocoUpgrading.value = false;
}

// === Scoop ===
async function loadScoop() {
  scoopLoading.value = true; scoopMsg.value = "";
  try {
    scoopInstalled.value = await invoke<boolean>("check_scoop");
    if (scoopInstalled.value) scoopList.value = await invoke<ScoopPackage[]>("list_scoop_upgrades");
  } catch { scoopInstalled.value = false; }
  scoopLoading.value = false;
}
async function upgradeScoop() {
  scoopUpgrading.value = true;
  scoopMsg.value = "Mise à jour en cours (scoop update + scoop update *)...";
  scoopUpgradeLog.value = "";
  // Écoute le log en temps réel
  if (scoopUnlisten) scoopUnlisten();
  scoopUnlisten = await listen<string>("scoop-upgrade-done", (ev) => {
    scoopUpgradeLog.value = ev.payload || "";
    scoopMsg.value = "Mise à jour Scoop terminée ✓";
    scoopUpgrading.value = false;
    loadScoop();
  });
  try {
    await invokeRaw("upgrade_scoop_all");
  } catch (e: any) {
    scoopMsg.value = "Erreur : " + e;
    scoopUpgrading.value = false;
  }
}

const chocoInstalling  = ref(false);
const chocoInstallMsg  = ref("");
const scoopInstalling  = ref(false);
const scoopInstallMsg  = ref("");

async function installWinget() {
  try {
    await invoke("open_url", { url: "ms-windows-store://pdp/?productid=9NBLGGH4NNS1" });
  } catch (e: any) {
    notify.error("Impossible d'ouvrir le Microsoft Store", String(e));
  }
}

async function installChoco() {
  chocoInstalling.value = true;
  chocoInstallMsg.value = "Installation Chocolatey en cours (droits admin requis)...";
  try {
    const msg = await invokeRaw<string>("install_package_manager", { manager: "chocolatey" });
    chocoInstallMsg.value = msg || "Chocolatey installé — relancez l'outil pour le détecter.";
    setTimeout(() => loadChoco(), 6000);
  } catch (e: any) {
    chocoInstallMsg.value = "Erreur : " + String(e);
    notify.error("Installation Chocolatey échouée", String(e));
  }
  chocoInstalling.value = false;
}

async function installScoop() {
  scoopInstalling.value = true;
  scoopInstallMsg.value = "Installation Scoop en cours (peut prendre 30s)...";
  try {
    const msg = await invokeRaw<string>("install_package_manager", { manager: "scoop" });
    scoopInstallMsg.value = msg || "Scoop installé — relancez l'outil pour le détecter.";
    setTimeout(() => loadScoop(), 6000);
  } catch (e: any) {
    scoopInstallMsg.value = "Erreur : " + String(e);
    notify.error("Installation Scoop échouée", String(e));
  }
  scoopInstalling.value = false;
}

// Filtres historique KB
const histSearch = ref('');
const histYearFilter = ref('');
const histSortDesc = ref(true);

const filteredHistory = computed(() => {
  let list = props.updatesHistory || [];
  if (histSearch.value) {
    const q = histSearch.value.toLowerCase();
    list = list.filter(u =>
      u.hotfix_id.toLowerCase().includes(q) ||
      u.description.toLowerCase().includes(q)
    );
  }
  if (histYearFilter.value) {
    list = list.filter(u => u.installed_on.includes(histYearFilter.value));
  }
  if (histSortDesc.value) {
    list = [...list].sort((a, b) =>
      new Date(b.installed_on || 0).getTime() - new Date(a.installed_on || 0).getTime()
    );
  }
  return list;
});

const histYears = computed(() => {
  const years = new Set<string>();
  (props.updatesHistory || []).forEach(u => {
    const m = (u.installed_on || '').match(/20\d\d/);
    if (m) years.add(m[0]);
  });
  return [...years].sort().reverse();
});

function exportHistory() {
  exportCSV((props.updatesHistory || []).map(u => ({
    KB: u.hotfix_id, Description: u.description,
    InstalleLe: u.installed_on, Par: u.installed_by || '',
  })), 'historique-mises-a-jour-' + new Date().toISOString().slice(0, 10));
}

function severityVariant(s: string) {
  const l = s.toLowerCase();
  if (l.includes("critical")) return "danger";
  if (l.includes("important")) return "warning";
  if (l.includes("moderate") || l.includes("low")) return "info";
  return "neutral";
}

onMounted(() => { loadWinget(); loadChoco(); loadScoop(); });
onUnmounted(() => { if (scoopUnlisten) scoopUnlisten(); });

async function launchSdi() {
  try {
    window.__nitrite_sdi_active = true;
    setTimeout(() => { window.__nitrite_sdi_active = false; }, 60000);
    await invoke("launch_sdi");
    notify.success("Snappy Driver Installer lancé");
  } catch (e) {
    window.__nitrite_sdi_active = false;
    notify.error("SDI introuvable", String(e));
  }
}
</script>

<template>
  <!-- ===== Snappy Driver ===== -->
  <div style="display:flex;justify-content:flex-end;margin-bottom:8px">
    <NButton variant="secondary" size="sm" @click="launchSdi" title="Lancer Snappy Driver Installer">
      <Zap :size="13" /> Snappy Driver Installer
    </NButton>
  </div>

  <!-- ===== Windows Update ===== -->
  <NCollapse title="Windows Update — Mises à jour disponibles" storageKey="diag-updates-wu" :defaultOpen="true">
    <div class="card-block">
      <div style="display:flex;gap:8px;flex-wrap:wrap;align-items:center;margin-bottom:10px">
        <NButton variant="primary" size="sm" :disabled="scanningWU" @click="scanWU">
          <NSpinner v-if="scanningWU" :size="12" />
          <RefreshCw v-else :size="13" />
          {{ scanningWU ? 'Scan en cours...' : 'Scanner les mises à jour' }}
        </NButton>
        <NButton v-if="wuScanned && pendingWU.length > 0" variant="success" size="sm"
          :disabled="triggeringWU" @click="triggerWU">
          <Download :size="13" />
          {{ triggeringWU ? 'Déclenchement...' : 'Installer toutes (' + pendingWU.length + ')' }}
        </NButton>
        <span v-if="wuMessage" class="muted" style="font-size:12px">{{ wuMessage }}</span>
      </div>

      <div v-if="!wuScanned && !scanningWU" class="diag-empty" style="padding:8px 0">
        Cliquez sur "Scanner" pour détecter les mises à jour Windows disponibles (utilise le cache local)
      </div>
      <div v-else-if="wuScanned && !pendingWU.length" style="display:flex;align-items:center;gap:8px;color:var(--success);font-size:13px">
        <CheckCircle :size="14" /> Windows est à jour — aucune mise à jour en attente
      </div>
      <div v-else-if="pendingWU.length" class="table-wrap">
        <table class="data-table">
          <thead><tr><th>Titre</th><th>KB</th><th>Sévérité</th><th>Taille</th><th>Téléchargé</th></tr></thead>
          <tbody>
            <tr v-for="(u, i) in pendingWU" :key="i">
              <td style="max-width:300px;white-space:normal;word-break:break-word;font-size:11px">{{ u.title }}</td>
              <td><code style="font-size:10px">{{ u.kb_ids || "—" }}</code></td>
              <td><NBadge :variant="severityVariant(u.severity)" style="font-size:10px">{{ u.severity }}</NBadge></td>
              <td class="muted" style="font-size:11px">{{ u.size_mb > 0 ? u.size_mb.toFixed(0)+' MB' : '—' }}</td>
              <td>
                <CheckCircle v-if="u.is_downloaded" :size="13" class="ic-ok" />
                <span v-else class="muted" style="font-size:11px">Non</span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </NCollapse>

  <!-- ===== Winget ===== -->
  <NCollapse title="WinGet — Gestionnaire de paquets Windows" storageKey="diag-updates-winget" :defaultOpen="false">
    <div class="card-block">
      <div v-if="wingetLoading" style="display:flex;align-items:center;gap:8px;font-size:13px"><NSpinner :size="14" /> Chargement...</div>
      <div v-else-if="!wingetOk" class="diag-empty" style="display:flex;flex-direction:column;gap:10px;align-items:flex-start">
        <span>WinGet non détecté ou non fonctionnel.</span>
        <NButton variant="primary" size="sm" @click="installWinget">
          <Download :size="13" /> Installer / Réparer WinGet (via Store)
        </NButton>
      </div>
      <template v-else>
        <div style="display:flex;gap:8px;align-items:center;flex-wrap:wrap;margin-bottom:10px">
          <NButton variant="ghost" size="sm" :disabled="wingetLoading" @click="loadWinget">
            <RefreshCw :size="12" /> Actualiser
          </NButton>
          <NButton v-if="wingetList.length > 0" variant="primary" size="sm"
            :disabled="wingetUpgrading" @click="upgradeWinget">
            <Download v-if="!wingetUpgrading" :size="13" />
            <NSpinner v-else :size="12" />
            {{ wingetUpgrading ? 'Mise à jour...' : 'Tout mettre à jour (' + wingetList.length + ')' }}
          </NButton>
          <span v-if="wingetMsg" class="muted" style="font-size:12px">{{ wingetMsg }}</span>
        </div>
        <div v-if="!wingetList.length" style="display:flex;align-items:center;gap:8px;color:var(--success);font-size:13px">
          <CheckCircle :size="14" /> Tous les paquets winget sont à jour
        </div>
        <div v-else class="table-wrap">
          <table class="data-table">
            <thead><tr><th>Nom</th><th>ID</th><th>Installé</th><th>Disponible</th></tr></thead>
            <tbody>
              <tr v-for="(p, i) in wingetList.slice(0, 50)" :key="i">
                <td>{{ p.name || "—" }}</td>
                <td><code style="font-size:10px">{{ p.id || "—" }}</code></td>
                <td class="muted" style="font-size:11px">{{ p.version || "—" }}</td>
                <td><NBadge variant="info" style="font-size:10px">{{ p.available || "—" }}</NBadge></td>
              </tr>
            </tbody>
          </table>
        </div>
      </template>
    </div>
  </NCollapse>

  <!-- ===== Chocolatey ===== -->
  <NCollapse title="Chocolatey" storageKey="diag-updates-chocolatey" :defaultOpen="false">
    <div class="card-block">
      <div v-if="chocoLoading" style="display:flex;align-items:center;gap:8px;font-size:13px"><NSpinner :size="14" /> Chargement...</div>
      <div v-else-if="!chocoInstalled" class="diag-empty" style="display:flex;flex-direction:column;gap:10px;align-items:flex-start">
        <span>Chocolatey n'est pas installé.</span>
        <NButton variant="primary" size="sm" :disabled="chocoInstalling" @click="installChoco">
          <NSpinner v-if="chocoInstalling" :size="12" />
          <Download v-else :size="13" />
          {{ chocoInstalling ? 'Installation...' : 'Installer Chocolatey' }}
        </NButton>
        <span v-if="chocoInstallMsg" class="muted" style="font-size:12px">{{ chocoInstallMsg }}</span>
      </div>
      <template v-else>
        <div style="display:flex;gap:8px;align-items:center;flex-wrap:wrap;margin-bottom:10px">
          <NButton variant="ghost" size="sm" @click="loadChoco"><RefreshCw :size="12" /> Actualiser</NButton>
          <NButton v-if="chocoList.length > 0" variant="primary" size="sm"
            :disabled="chocoUpgrading" @click="upgradeChoco">
            <Download v-if="!chocoUpgrading" :size="13" />
            <NSpinner v-else :size="12" />
            {{ chocoUpgrading ? 'Mise à jour...' : 'Tout mettre à jour (' + chocoList.length + ')' }}
          </NButton>
          <span v-if="chocoMsg" class="muted" style="font-size:12px">{{ chocoMsg }}</span>
        </div>
        <div v-if="!chocoList.length" style="display:flex;align-items:center;gap:8px;color:var(--success);font-size:13px">
          <CheckCircle :size="14" /> Tous les paquets Chocolatey sont à jour
        </div>
        <div v-else class="table-wrap">
          <table class="data-table">
            <thead><tr><th>Paquet</th><th>Installé</th><th>Disponible</th></tr></thead>
            <tbody>
              <tr v-for="(p, i) in chocoList" :key="i">
                <td><Package :size="12" style="margin-right:4px;opacity:.6" />{{ p.name || "—" }}</td>
                <td class="muted" style="font-size:11px">{{ p.current_version || "—" }}</td>
                <td><NBadge variant="info" style="font-size:10px">{{ p.available_version || "—" }}</NBadge></td>
              </tr>
            </tbody>
          </table>
        </div>
      </template>
    </div>
  </NCollapse>

  <!-- ===== Scoop ===== -->
  <NCollapse title="Scoop — Gestionnaire de paquets" storageKey="diag-updates-scoop" :defaultOpen="false">
    <div class="card-block">
      <div v-if="scoopLoading" style="display:flex;align-items:center;gap:8px;font-size:13px">
        <NSpinner :size="14" /> Détection Scoop en cours...
      </div>

      <!-- Non installé -->
      <div v-else-if="!scoopInstalled" style="display:flex;flex-direction:column;gap:12px">
        <div style="display:flex;align-items:center;gap:8px;font-size:13px;color:var(--text-secondary)">
          <AlertTriangle :size="14" class="ic-warn" />
          Scoop n'est pas installé sur ce PC.
        </div>
        <div style="font-size:12px;color:var(--text-muted);padding:8px 12px;background:var(--bg-secondary);border-radius:6px;border:1px solid var(--border)">
          Scoop est un gestionnaire de paquets en ligne de commande pour Windows.
          L'installation se fait sans droits administrateur.
        </div>
        <div style="display:flex;align-items:center;gap:8px;flex-wrap:wrap">
          <NButton variant="primary" size="sm" :disabled="scoopInstalling" @click="installScoop">
            <NSpinner v-if="scoopInstalling" :size="12" />
            <Download v-else :size="13" />
            {{ scoopInstalling ? 'Installation en cours (30-60s)...' : 'Installer Scoop' }}
          </NButton>
          <NButton variant="ghost" size="sm" :disabled="scoopLoading" @click="loadScoop">
            <RefreshCw :size="12" /> Vérifier à nouveau
          </NButton>
        </div>
        <div v-if="scoopInstallMsg"
          style="font-size:12px;padding:8px 10px;border-radius:5px;border-left:3px solid var(--accent-primary);background:var(--bg-secondary)">
          {{ scoopInstallMsg }}
        </div>
      </div>

      <!-- Installé -->
      <template v-else>
        <div style="display:flex;gap:8px;align-items:center;flex-wrap:wrap;margin-bottom:10px">
          <NButton variant="ghost" size="sm" :disabled="scoopLoading" @click="loadScoop">
            <RefreshCw :size="12" /> Actualiser
          </NButton>
          <NButton v-if="scoopList.length > 0" variant="primary" size="sm"
            :disabled="scoopUpgrading" @click="upgradeScoop">
            <NSpinner v-if="scoopUpgrading" :size="12" />
            <Download v-else :size="13" />
            {{ scoopUpgrading ? 'Mise à jour en cours...' : `Tout mettre à jour (${scoopList.length})` }}
          </NButton>
          <NButton v-else variant="ghost" size="sm" :disabled="scoopUpgrading" @click="upgradeScoop">
            <RefreshCw :size="12" /> Vérifier les mises à jour
          </NButton>
          <span v-if="scoopMsg" :class="scoopMsg.includes('✓') ? 'ic-ok' : 'muted'" style="font-size:12px">
            {{ scoopMsg }}
          </span>
        </div>

        <div v-if="!scoopList.length && !scoopUpgrading && !scoopUpgradeLog"
          style="display:flex;align-items:center;gap:8px;color:var(--success);font-size:13px">
          <CheckCircle :size="14" /> Tous les paquets Scoop sont à jour
        </div>

        <div v-else-if="scoopList.length" class="table-wrap">
          <table class="data-table">
            <thead><tr><th>Paquet</th><th>Version installée</th><th>Disponible</th></tr></thead>
            <tbody>
              <tr v-for="(p, i) in scoopList" :key="i">
                <td style="font-weight:500"><Package :size="12" style="margin-right:5px;opacity:.5" />{{ p.name }}</td>
                <td class="muted" style="font-size:11px">{{ p.installed || "—" }}</td>
                <td><NBadge variant="info" style="font-size:10px">{{ p.available || "—" }}</NBadge></td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- Log de mise à jour -->
        <div v-if="scoopUpgradeLog" style="margin-top:10px">
          <p style="font-size:11px;color:var(--text-secondary);margin-bottom:4px">Résultat :</p>
          <pre style="font-size:10px;background:var(--bg-tertiary,#0d0d0f);border:1px solid var(--border);border-radius:4px;padding:8px;max-height:160px;overflow-y:auto;white-space:pre-wrap;color:var(--text-secondary)">{{ scoopUpgradeLog }}</pre>
        </div>
      </template>
    </div>
  </NCollapse>

  <!-- ===== Historique KB ===== -->
  <NCollapse :title="'Historique des mises à jour installées — ' + updatesHistory.length" storageKey="diag-updates-history" :defaultOpen="false">
    <div v-if="!updatesHistory.length" class="diag-empty">Chargement de l'historique...</div>
    <div v-else class="card-block">
      <!-- Filtres historique -->
      <div style="display:flex;gap:8px;flex-wrap:wrap;margin-bottom:10px;align-items:center">
        <input v-model="histSearch" placeholder="Rechercher KB, description..."
          style="flex:1;min-width:180px;padding:5px 10px;background:var(--bg-tertiary);border:1px solid var(--border);border-radius:6px;color:var(--text-primary);font-size:12px" />
        <select v-model="histYearFilter"
          style="padding:5px 8px;background:var(--bg-tertiary);border:1px solid var(--border);border-radius:6px;color:var(--text-primary);font-size:12px">
          <option value="">Toutes les années</option>
          <option v-for="y in histYears" :key="y" :value="y">{{ y }}</option>
        </select>
        <button @click="histSortDesc = !histSortDesc"
          style="font-size:11px;padding:4px 10px;border:1px solid var(--border);border-radius:6px;background:var(--bg-secondary);color:var(--text-secondary);cursor:pointer">
          Tri: {{ histSortDesc ? 'Plus récent ↓' : 'Plus ancien ↑' }}
        </button>
        <NButton variant="ghost" size="sm" @click="exportHistory">↓ CSV</NButton>
        <span class="muted" style="font-size:11px">{{ filteredHistory.length }} / {{ (props.updatesHistory||[]).length }}</span>
      </div>
      <div class="table-wrap">
        <table class="data-table">
          <thead><tr><th>KB ID</th><th>Description</th><th>Installé le</th><th>Par</th></tr></thead>
          <tbody>
            <tr v-for="(u, i) in filteredHistory.slice(0, 300)" :key="i">
              <td><code>{{ u.hotfix_id }}</code></td>
              <td>{{ u.description }}</td>
              <td class="muted">{{ u.installed_on }}</td>
              <td class="muted">{{ u.installed_by || "—" }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </NCollapse>
</template>
