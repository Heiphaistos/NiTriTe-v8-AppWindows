<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@/utils/invoke";
import { cachedInvoke, refreshCached } from "@/composables/useCachedInvoke";
import NCard from "@/components/ui/NCard.vue";
import NButton from "@/components/ui/NButton.vue";
import NBadge from "@/components/ui/NBadge.vue";
import NSkeleton from "@/components/ui/NSkeleton.vue";
import { useNotificationStore } from "@/stores/notifications";
import { Shield, RefreshCw, Plus, Clock, CheckCircle2, AlertTriangle, RotateCcw, Trash2, User, Bot } from "lucide-vue-next";

const notify = useNotificationStore();

interface RestorePoint {
  sequence_number: number;
  description: string;
  creation_time: string;
  restore_type: string;
  id?: string;
  size_bytes?: number;
}

const points = ref<RestorePoint[]>([]);
const loading = ref(true);
const creating = ref(false);
const newDesc = ref("");
const restoringId = ref<number | null>(null);
const deletingId = ref<number | null>(null);

// Le point le plus récent (numéro le plus élevé après tri)
const latestSequence = computed(() => {
  if (!points.value.length) return -1;
  const nums = points.value.map(p => p.sequence_number).filter(n => typeof n === "number" && !isNaN(n));
  return nums.length ? Math.max(...nums) : -1;
});

function isManual(t: string): boolean {
  return !["APPLICATION_INSTALL", "APPLICATION_UNINSTALL", "DEVICE_DRIVER_INSTALL",
    "MODIFY_SETTINGS", "CANCELLED_OPERATION", "RESTORE"].includes(t);
}

function formatType(t: string) {
  const map: Record<string, string> = {
    APPLICATION_INSTALL: "Installation App",
    APPLICATION_UNINSTALL: "Désinstallation",
    DEVICE_DRIVER_INSTALL: "Driver",
    MODIFY_SETTINGS: "Paramètres",
    CANCELLED_OPERATION: "Annulé",
    RESTORE: "Restauration",
  };
  return map[t] ?? t;
}

function typeVariant(t: string): "success" | "info" | "warning" | "neutral" {
  if (t.includes("INSTALL")) return "success";
  if (t.includes("DRIVER")) return "info";
  if (t.includes("UNINSTALL")) return "warning";
  return "neutral";
}

function formatDate(raw: string): string {
  if (!raw) return "—";
  try {
    const m = raw.match(/^(\d{4})(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})/);
    if (m) {
      return `${m[3]}/${m[2]}/${m[1]} ${m[4]}:${m[5]}`;
    }
    return new Date(raw).toLocaleString("fr-FR", { dateStyle: "short", timeStyle: "short" });
  } catch {
    return raw;
  }
}

function formatSize(bytes?: number): string {
  if (bytes == null || bytes === 0) return "";
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} Ko`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} Mo`;
}

async function load() {
  loading.value = true;
  try {
    points.value = await cachedInvoke<RestorePoint[]>("list_restore_points_cmd");
    points.value.sort((a, b) => b.sequence_number - a.sequence_number);
  } catch (e: unknown) {
    notify.error("Erreur", String(e));
  }
  loading.value = false;
}

async function create() {
  const desc = newDesc.value.trim() || "Point de restauration NiTriTe";
  creating.value = true;
  try {
    await invoke("create_restore_point_cmd", { description: desc });
    notify.success("Point créé", desc);
    newDesc.value = "";
    await load();
  } catch (e: unknown) {
    notify.error("Erreur création", String(e));
  }
  creating.value = false;
}

async function restorePoint(p: RestorePoint) {
  const ok = confirm(
    `Restaurer le système à ce point ?\n\n"${p.description}" (#${p.sequence_number})\n\nL'ordinateur va redémarrer.`
  );
  if (!ok) return;
  restoringId.value = p.sequence_number;
  try {
    await invoke("run_system_command", {
      cmd: "powershell",
      args: ["-Command", `Restore-Computer -RestorePoint ${p.sequence_number} -Confirm:$false`],
    });
    notify.success("Restauration lancée", "Le système va redémarrer.");
  } catch (e: unknown) {
    notify.error("Erreur restauration", String(e));
  }
  restoringId.value = null;
}

async function deletePoint(p: RestorePoint) {
  const ok = confirm(
    `Supprimer ce point de restauration ?\n\n"${p.description}" (#${p.sequence_number})\n\nCette action est irréversible.`
  );
  if (!ok) return;
  deletingId.value = p.sequence_number;
  try {
    const shadowArg = p.id ? `/shadow=${p.id}` : `/oldest`;
    await invoke("run_system_command", {
      cmd: "vssadmin",
      args: ["delete", "shadows", shadowArg, "/quiet"],
    });
    notify.success("Point supprimé", `#${p.sequence_number}`);
    await load();
  } catch (e: unknown) {
    notify.error("Erreur suppression", String(e));
  }
  deletingId.value = null;
}

onMounted(load);
</script>

<template>
  <div class="rp-page">
    <div class="page-header">
      <div class="header-icon"><Shield :size="22" /></div>
      <div>
        <h1>Points de Restauration</h1>
        <p class="subtitle">Créer et consulter les points de restauration système Windows</p>
      </div>
      <NButton variant="ghost" size="sm" :loading="loading" @click="load" style="margin-left:auto">
        <RefreshCw :size="13" /> Actualiser
      </NButton>
    </div>

    <div class="warning-banner">
      <AlertTriangle :size="14" />
      <span>La création d'un point de restauration nécessite les droits administrateur et la Protection du Système activée.</span>
    </div>

    <!-- Créer un point -->
    <NCard>
      <template #header>
        <div class="section-header"><Plus :size="15" /><span>Créer un point de restauration</span></div>
      </template>
      <div class="create-row">
        <input
          v-model="newDesc"
          class="desc-input"
          placeholder="Description (ex: Avant installation driver)"
          @keydown.enter="create"
        />
        <NButton variant="primary" size="sm" :loading="creating" @click="create">
          <Plus :size="13" /> Créer
        </NButton>
      </div>
    </NCard>

    <!-- Liste -->
    <NCard>
      <template #header>
        <div class="section-header">
          <Shield :size="15" />
          <span>Points existants ({{ points.length }})</span>
        </div>
      </template>

      <div v-if="loading">
        <NSkeleton v-for="i in 4" :key="i" height="52px" style="margin-bottom:6px" />
      </div>

      <div v-else-if="!points.length" class="empty-state">
        <Shield :size="32" style="opacity:.2" />
        <p>Aucun point de restauration trouvé</p>
        <p class="empty-hint">Activez la Protection du Système dans les propriétés système Windows</p>
      </div>

      <div v-else class="points-list">
        <div v-for="p in points" :key="p.sequence_number" class="point-row">
          <div class="point-num">#{{ p.sequence_number }}</div>
          <div class="point-info">
            <div class="point-desc-row">
              <span class="point-desc">{{ p.description || 'Sans description' }}</span>
              <!-- Badge Auto / Manuel -->
              <NBadge :variant="isManual(p.restore_type) ? 'accent' : 'neutral'" size="sm" class="type-mode-badge">
                <component :is="isManual(p.restore_type) ? User : Bot" :size="10" />
                {{ isManual(p.restore_type) ? 'Manuel' : 'Auto' }}
              </NBadge>
            </div>
            <div class="point-meta">
              <span class="point-date"><Clock :size="11" /> {{ formatDate(p.creation_time) }}</span>
              <span v-if="p.size_bytes" class="point-size">{{ formatSize(p.size_bytes) }}</span>
            </div>
          </div>

          <NBadge :variant="typeVariant(p.restore_type)" size="sm">
            {{ formatType(p.restore_type) }}
          </NBadge>

          <!-- Bouton Restaurer -->
          <NButton
            variant="secondary"
            size="sm"
            :loading="restoringId === p.sequence_number"
            @click="restorePoint(p)"
            title="Restaurer le système à ce point"
          >
            <RotateCcw :size="12" />
            Restaurer
          </NButton>

          <!-- Bouton Supprimer (sauf le plus récent) -->
          <NButton
            v-if="p.sequence_number !== latestSequence"
            variant="ghost"
            size="sm"
            :loading="deletingId === p.sequence_number"
            @click="deletePoint(p)"
            title="Supprimer ce point"
            style="color: var(--danger)"
          >
            <Trash2 :size="12" />
          </NButton>

          <CheckCircle2 :size="16" style="color:var(--success);flex-shrink:0" />
        </div>
      </div>
    </NCard>
  </div>
</template>

<style scoped>
.rp-page { display: flex; flex-direction: column; gap: 14px; }
.page-header { display: flex; align-items: center; gap: 12px; }
.header-icon { width: 42px; height: 42px; border-radius: var(--radius-lg); background: var(--success-muted); display: flex; align-items: center; justify-content: center; color: var(--success); flex-shrink: 0; }
h1 { font-size: 22px; font-weight: 700; }
.subtitle { font-size: 12px; color: var(--text-muted); }
.warning-banner { display: flex; gap: 10px; align-items: center; padding: 10px 14px; background: var(--warning-muted); border: 1px solid color-mix(in srgb, var(--warning) 30%, transparent); border-radius: var(--radius-md); font-size: 12px; color: var(--warning); }
.section-header { display: flex; align-items: center; gap: 8px; }
.create-row { display: flex; gap: 10px; align-items: center; }
.desc-input { flex: 1; padding: 8px 12px; background: var(--bg-tertiary); border: 1px solid var(--border); border-radius: var(--radius-md); color: var(--text-primary); font-size: 13px; font-family: inherit; outline: none; transition: border-color .15s; }
.desc-input:focus { border-color: var(--accent-primary); }
.points-list { display: flex; flex-direction: column; gap: 4px; }
.point-row { display: flex; align-items: center; gap: 10px; padding: 10px 14px; background: var(--bg-tertiary); border-radius: var(--radius-md); border: 1px solid var(--border); transition: border-color .15s; flex-wrap: wrap; }
.point-row:hover { border-color: var(--border-hover); }
.point-num { font-family: "JetBrains Mono", monospace; font-size: 11px; color: var(--text-muted); width: 28px; flex-shrink: 0; }
.point-info { flex: 1; display: flex; flex-direction: column; gap: 3px; min-width: 0; }
.point-desc-row { display: flex; align-items: center; gap: 8px; }
.point-desc { font-size: 13px; font-weight: 500; color: var(--text-primary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.type-mode-badge { display: flex; align-items: center; gap: 3px; flex-shrink: 0; }
.point-meta { display: flex; align-items: center; gap: 12px; }
.point-date { display: flex; align-items: center; gap: 4px; font-size: 11px; color: var(--text-muted); }
.point-size { font-size: 11px; color: var(--text-muted); font-family: "JetBrains Mono", monospace; }
.empty-state { display: flex; flex-direction: column; align-items: center; gap: 8px; padding: 40px; color: var(--text-muted); font-size: 13px; text-align: center; }
.empty-hint { font-size: 11px; color: var(--text-muted); opacity: .7; }
</style>
