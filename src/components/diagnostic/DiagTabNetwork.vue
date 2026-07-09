<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { invoke } from "@/utils/invoke";
import type { NetworkAdapter, TcpConnection, WifiInfo, NetworkExtended } from "@/types/diagnostic";
import {
  Search, Wifi, WifiOff, Globe, Shield, ShieldOff, Router, Share2,
  Activity, Server, Database, RefreshCw, CheckCircle, XCircle, MapPin,
} from "lucide-vue-next";
import NBadge from "@/components/ui/NBadge.vue";
import NButton from "@/components/ui/NButton.vue";
import NSpinner from "@/components/ui/NSpinner.vue";
import DiagBanner from "@/components/ui/DiagBanner.vue";
import NCollapse from "@/components/ui/NCollapse.vue";

const props = defineProps<{
  tab: string;
  networkAdapters: NetworkAdapter[];
  connections: TcpConnection[];
  wifiInfo: WifiInfo | null;
}>();

// === Connexions ===
const connSearch = ref("");
const connFilter = ref("all");
const filteredConns = computed(() => {
  let list = props.connections;
  if (connFilter.value !== "all") list = list.filter(c => c.state === connFilter.value);
  if (connSearch.value) {
    const q = connSearch.value.toLowerCase();
    list = list.filter(c =>
      (c.process_name || "").toLowerCase().includes(q) ||
      (c.local_address || "").includes(q) ||
      (c.remote_address || "").includes(q)
    );
  }
  return list;
});
const stateVariant = (s: string) => {
  if (s === "Established") return "success";
  if (s === "Listen") return "info";
  if (s === "TimeWait") return "warning";
  return "default";
};

// === Extended data (réseau étendu) ===
const extLoading = ref(false);
const extData = ref<NetworkExtended | null>(null);

async function loadExtended() {
  extLoading.value = true;
  try { extData.value = await invoke<NetworkExtended>("get_network_extended"); }
  catch { extData.value = null; }
  extLoading.value = false;
}

onMounted(() => { if (props.tab === "network") loadExtended(); });
watch(() => props.tab, (t) => { if (t === "network" && !extData.value && !extLoading.value) loadExtended(); });

// Helpers
function fmtBytes(b: number): string {
  if (!b || b === 0) return "0 B";
  if (b < 1024) return `${b} B`;
  if (b < 1048576) return `${(b / 1024).toFixed(1)} KB`;
  if (b < 1073741824) return `${(b / 1048576).toFixed(1)} MB`;
  return `${(b / 1073741824).toFixed(2)} GB`;
}
type BadgeVariant = "default" | "success" | "warning" | "info" | "accent" | "danger" | "neutral";
function pingVariant(ok: boolean, ms: number): BadgeVariant {
  if (!ok) return "danger";
  if (ms < 20) return "success";
  if (ms < 80) return "info";
  return "warning";
}
</script>

<template>
  <!-- ===== Adaptateurs réseau ===== -->
  <template v-if="tab === 'network'">
    <div class="diag-tab-content">
      <DiagBanner :icon="Globe" title="Adaptateurs Réseau" desc="Interfaces, IP, DNS, DHCP et diagnostics avancés" color="cyan" />
      <div v-if="!networkAdapters.length" class="diag-empty">Aucun adaptateur réseau actif</div>
      <template v-else>

        <!-- WiFi Block -->
        <NCollapse
          v-if="wifiInfo"
          :title="`WiFi actif — ${wifiInfo.ssid || 'N/A'}`"
          storageKey="diag-net-wifi"
          defaultOpen
        >
          <div class="card-block" style="border:1px solid var(--accent)">
            <div class="block-title">
              <Wifi :size="16" style="color:var(--accent)" />
              <span>{{ wifiInfo.ssid || "N/A" }}</span>
              <NBadge variant="success">{{ wifiInfo.signal_percent }}%</NBadge>
              <NBadge variant="info">{{ wifiInfo.band }}</NBadge>
              <NBadge variant="default">{{ wifiInfo.authentication }}</NBadge>
            </div>
            <div class="info-grid">
              <div class="info-row"><span>BSSID (MAC Point d'accès)</span><code>{{ wifiInfo.bssid || "N/A" }}</code></div>
              <div class="info-row"><span>Signal</span><span>{{ wifiInfo.signal_percent }}%</span></div>
              <div class="info-row"><span>Bande</span><NBadge variant="info">{{ wifiInfo.band }}</NBadge></div>
              <div class="info-row"><span>Canal</span><span>{{ wifiInfo.channel > 0 ? wifiInfo.channel : 'N/A' }}</span></div>
              <div class="info-row"><span>Protocole radio</span><span>{{ wifiInfo.protocol }}</span></div>
              <div class="info-row"><span>Chiffrement</span><span>{{ wifiInfo.security || "N/A" }}</span></div>
              <div class="info-row"><span>Authentification</span><span>{{ wifiInfo.authentication || "N/A" }}</span></div>
              <div class="info-row"><span>Débit réception</span><span>{{ wifiInfo.receive_rate_mbps > 0 ? wifiInfo.receive_rate_mbps + ' Mbps' : 'N/A' }}</span></div>
              <div class="info-row"><span>Débit émission</span><span>{{ wifiInfo.transmit_rate_mbps > 0 ? wifiInfo.transmit_rate_mbps + ' Mbps' : 'N/A' }}</span></div>
              <div class="info-row"><span>Adaptateur</span><span>{{ wifiInfo.adapter_name }}</span></div>
            </div>
          </div>
        </NCollapse>
        <div v-else class="card-block" style="display:flex;align-items:center;gap:8px;margin-bottom:16px;color:var(--text-secondary)">
          <WifiOff :size="14" /><span style="font-size:13px">Aucune connexion WiFi active</span>
        </div>

        <!-- Adaptateurs -->
        <NCollapse
          :title="`Adaptateurs réseau — ${networkAdapters.length}`"
          storageKey="diag-net-adapters"
          defaultOpen
        >
          <div v-for="(a, i) in networkAdapters" :key="i" class="card-block">
            <div class="block-title">
              <span>{{ a.name }}</span>
              <span class="muted" style="font-size:12px;font-weight:400">{{ a.description }}</span>
              <NBadge :variant="a.status === 'Up' || a.status === 'Connected' ? 'success' : 'warning'">{{ a.status }}</NBadge>
              <NBadge v-if="a.is_physical" variant="info">Physique</NBadge>
            </div>
            <div class="info-grid">
              <div class="info-row"><span>Adresse MAC</span><code>{{ a.mac_address || "N/A" }}</code></div>
              <div class="info-row"><span>Vitesse</span><span>{{ a.speed_mbps > 0 ? a.speed_mbps + ' Mbps' : 'N/A' }}</span></div>
              <div class="info-row info-full"><span>Adresses IP</span>
                <div style="display:flex;flex-wrap:wrap;gap:4px;justify-content:flex-end">
                  <code v-for="ip in a.ip_addresses" :key="ip">{{ ip }}</code>
                  <span v-if="!a.ip_addresses.length" class="muted">N/A</span>
                </div>
              </div>
              <div class="info-row"><span>Passerelle par défaut</span><code>{{ a.default_gateway?.join(", ") || "N/A" }}</code></div>
              <div class="info-row"><span>Serveurs DNS</span>
                <div style="display:flex;flex-wrap:wrap;gap:4px;justify-content:flex-end">
                  <code v-for="dns in (a.dns_servers || []).slice(0,4)" :key="dns" style="font-size:11px">{{ dns }}</code>
                </div>
              </div>
              <div class="info-row"><span>DHCP</span>
                <NBadge :variant="a.dhcp_enabled ? 'info' : 'default'">{{ a.dhcp_enabled ? "Activé (auto)" : "Manuel (statique)" }}</NBadge>
              </div>
            </div>
          </div>
        </NCollapse>

        <!-- Diagnostics réseau avancés -->
        <NCollapse title="Diagnostics réseau avancés" storageKey="diag-net-extended" defaultOpen>
          <template #header-extra>
            <NButton variant="ghost" size="sm" :disabled="extLoading" @click.stop="loadExtended">
              <NSpinner v-if="extLoading" :size="12" />
              <RefreshCw v-else :size="12" />
              {{ extLoading ? 'Analyse...' : 'Actualiser' }}
            </NButton>
          </template>

          <div v-if="extLoading" style="display:flex;align-items:center;gap:8px;font-size:13px;padding:12px 0">
            <NSpinner :size="14" /> Analyse réseau en cours (ping, ARP, routage, proxy...)
          </div>

          <template v-else-if="extData">
            <!-- Tests de connectivité -->
            <NCollapse title="Tests de connectivité (ping)" storageKey="diag-net-ping" defaultOpen variant="subtle">
              <div class="card-block">
                <div class="info-grid">
                  <div class="info-row">
                    <span style="display:flex;align-items:center;gap:4px">
                      <Router :size="12" /> Passerelle par défaut
                    </span>
                    <div v-if="extData.ping_gateway">
                      <NBadge :variant="pingVariant(extData.ping_gateway.success, extData.ping_gateway.avg)">
                        {{ extData.ping_gateway.host }}
                        {{ extData.ping_gateway.success ? '— ' + extData.ping_gateway.avg + ' ms' : '— Injoignable' }}
                      </NBadge>
                    </div>
                    <span v-else class="muted">N/A</span>
                  </div>
                  <div class="info-row">
                    <span style="display:flex;align-items:center;gap:4px"><Globe :size="12" /> 8.8.8.8 (Google DNS)</span>
                    <NBadge v-if="extData.ping_google" :variant="pingVariant(extData.ping_google.success, extData.ping_google.avg)">
                      {{ extData.ping_google.success ? extData.ping_google.avg + ' ms' : 'Injoignable' }}
                    </NBadge>
                  </div>
                  <div class="info-row">
                    <span style="display:flex;align-items:center;gap:4px"><Globe :size="12" /> 1.1.1.1 (Cloudflare)</span>
                    <NBadge v-if="extData.ping_cloudflare" :variant="pingVariant(extData.ping_cloudflare.success, extData.ping_cloudflare.avg)">
                      {{ extData.ping_cloudflare.success ? extData.ping_cloudflare.avg + ' ms' : 'Injoignable' }}
                    </NBadge>
                  </div>
                  <div class="info-row">
                    <span style="display:flex;align-items:center;gap:4px"><MapPin :size="12" /> IP publique</span>
                    <code v-if="extData.public_ip" style="color:var(--accent)">{{ extData.public_ip }}</code>
                    <span v-else class="muted">Non disponible</span>
                  </div>
                </div>
              </div>
            </NCollapse>

            <!-- Pare-feu -->
            <NCollapse title="Pare-feu Windows" storageKey="diag-net-firewall" defaultOpen variant="subtle">
              <div v-if="extData.firewall" class="card-block">
                <div class="info-grid">
                  <div class="info-row"><span>Profil Domaine</span>
                    <NBadge :variant="extData.firewall.domain ? 'success' : 'danger'">
                      <component :is="extData.firewall.domain ? CheckCircle : XCircle" :size="11" style="margin-right:3px" />
                      {{ extData.firewall.domain ? 'Activé' : 'Désactivé ⚠' }}
                    </NBadge>
                  </div>
                  <div class="info-row"><span>Profil Privé</span>
                    <NBadge :variant="extData.firewall.private ? 'success' : 'danger'">
                      {{ extData.firewall.private ? 'Activé' : 'Désactivé ⚠' }}
                    </NBadge>
                  </div>
                  <div class="info-row"><span>Profil Public</span>
                    <NBadge :variant="extData.firewall.public ? 'success' : 'danger'">
                      {{ extData.firewall.public ? 'Activé' : 'Désactivé ⚠' }}
                    </NBadge>
                  </div>
                </div>
              </div>
            </NCollapse>

            <!-- Proxy -->
            <NCollapse title="Proxy système" storageKey="diag-net-proxy" defaultOpen variant="subtle">
              <div v-if="extData.proxy" class="card-block">
                <div class="info-grid">
                  <div class="info-row"><span>Proxy actif</span>
                    <NBadge :variant="extData.proxy.enabled ? 'warning' : 'success'">
                      {{ extData.proxy.enabled ? 'Oui' : 'Non' }}
                    </NBadge>
                  </div>
                  <div v-if="extData.proxy.enabled" class="info-row">
                    <span>Serveur proxy</span><code>{{ extData.proxy.server || "—" }}</code>
                  </div>
                  <div v-if="extData.proxy.enabled && extData.proxy.bypass" class="info-row info-full">
                    <span>Exceptions</span><span style="font-size:12px;color:var(--text-secondary)">{{ extData.proxy.bypass }}</span>
                  </div>
                </div>
              </div>
            </NCollapse>

            <!-- Test DNS -->
            <NCollapse title="Test de résolution DNS (google.com)" storageKey="diag-net-dns" :defaultOpen="false" variant="subtle">
              <div class="card-block">
                <div v-if="!extData.dns_test?.length" class="diag-empty">Résolution DNS échouée ou non disponible</div>
                <div v-else class="table-wrap">
                  <table class="data-table">
                    <thead><tr><th>Nom</th><th>Type</th><th>IP résolue</th></tr></thead>
                    <tbody>
                      <tr v-for="(r, i) in extData.dns_test" :key="i">
                        <td>{{ r.name || "—" }}</td>
                        <td><NBadge variant="info" style="font-size:10px">{{ r.type || "—" }}</NBadge></td>
                        <td><code style="font-size:11px">{{ r.ip || "—" }}</code></td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </NCollapse>

            <!-- Stats réseau -->
            <NCollapse title="Statistiques par adaptateur (depuis démarrage)" storageKey="diag-net-stats" :defaultOpen="false" variant="subtle">
              <div class="card-block">
                <div v-if="!extData.stats?.length" class="diag-empty">Aucune statistique disponible</div>
                <div v-else class="table-wrap">
                  <table class="data-table">
                    <thead><tr><th>Adaptateur</th><th>Reçu</th><th>Envoyé</th></tr></thead>
                    <tbody>
                      <tr v-for="(s, i) in extData.stats" :key="i">
                        <td style="font-weight:500">{{ s.name }}</td>
                        <td><code style="color:var(--success)">{{ fmtBytes(s.recv_bytes) }}</code></td>
                        <td><code style="color:var(--accent)">{{ fmtBytes(s.sent_bytes) }}</code></td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </NCollapse>

            <!-- Table ARP -->
            <NCollapse title="Table ARP — correspondances IP ↔ MAC" storageKey="diag-net-arp" :defaultOpen="false" variant="subtle">
              <div class="card-block">
                <div v-if="!extData.arp_table?.length" class="diag-empty">Table ARP vide</div>
                <div v-else class="table-wrap">
                  <table class="data-table">
                    <thead><tr><th>Adresse IP</th><th>Adresse MAC</th><th>Type</th></tr></thead>
                    <tbody>
                      <tr v-for="(e, i) in extData.arp_table.slice(0, 80)" :key="i">
                        <td><code>{{ e.ip }}</code></td>
                        <td><code style="color:var(--text-secondary);font-size:11px">{{ e.mac }}</code></td>
                        <td><NBadge :variant="e.type === 'dynamic' ? 'info' : 'default'" style="font-size:10px">{{ e.type }}</NBadge></td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </NCollapse>

            <!-- Table de routage -->
            <NCollapse title="Table de routage IPv4" storageKey="diag-net-routes" :defaultOpen="false" variant="subtle">
              <div class="card-block">
                <div v-if="!extData.routes?.length" class="diag-empty">Aucune route trouvée</div>
                <div v-else class="table-wrap">
                  <table class="data-table">
                    <thead><tr><th>Destination</th><th>Passerelle</th><th>Métrique</th><th>Interface</th></tr></thead>
                    <tbody>
                      <tr v-for="(r, i) in extData.routes.slice(0, 50)" :key="i">
                        <td><code style="font-size:11px">{{ r.prefix }}</code></td>
                        <td><code style="font-size:12px;color:var(--text-secondary)">{{ r.next_hop }}</code></td>
                        <td class="muted">{{ r.metric }}</td>
                        <td style="font-size:11px;max-width:150px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">{{ r.iface }}</td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </NCollapse>

            <!-- Partages réseau -->
            <NCollapse title="Partages réseau (SMB)" storageKey="diag-net-shares" :defaultOpen="false" variant="subtle">
              <div class="card-block">
                <div v-if="!extData.shares?.length" class="diag-empty">Aucun partage réseau configuré</div>
                <div v-else class="table-wrap">
                  <table class="data-table">
                    <thead><tr><th>Nom</th><th>Chemin</th><th>Description</th></tr></thead>
                    <tbody>
                      <tr v-for="(s, i) in extData.shares" :key="i">
                        <td style="font-weight:500"><Share2 :size="11" style="margin-right:4px;opacity:.5" />{{ s.name }}</td>
                        <td><code style="font-size:11px">{{ s.path || "—" }}</code></td>
                        <td class="muted" style="font-size:11px">{{ s.desc || "—" }}</td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </NCollapse>

            <!-- Fichier hosts -->
            <NCollapse title="Fichier hosts" storageKey="diag-net-hosts" :defaultOpen="false" variant="subtle">
              <div class="card-block">
                <div v-if="!extData.hosts_entries?.length" class="diag-empty">Fichier hosts vide ou non accessible</div>
                <div v-else class="table-wrap">
                  <table class="data-table">
                    <thead><tr><th>IP</th><th>Hôte</th></tr></thead>
                    <tbody>
                      <tr v-for="(h, i) in extData.hosts_entries" :key="i">
                        <td><code>{{ h.ip }}</code></td>
                        <td><code style="font-size:11px">{{ h.host }}</code></td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </NCollapse>

            <!-- Réseaux WiFi disponibles -->
            <NCollapse
              v-if="extData.wifi_networks?.length"
              :title="`Réseaux WiFi disponibles — ${extData.wifi_networks.length}`"
              storageKey="diag-net-wifi-scan"
              :defaultOpen="false"
              variant="subtle"
            >
              <div class="card-block">
                <div style="display:flex;flex-wrap:wrap;gap:6px">
                  <NBadge v-for="(ssid, i) in extData.wifi_networks" :key="i" variant="default" style="font-size:11px">
                    <Wifi :size="10" style="margin-right:3px" />{{ ssid || "(masqué)" }}
                  </NBadge>
                </div>
              </div>
            </NCollapse>
          </template>
        </NCollapse>
      </template>
    </div>
  </template>

  <!-- ===== Connexions TCP/UDP actives ===== -->
  <template v-else-if="tab === 'connections'">
    <div class="diag-tab-content">
      <DiagBanner :icon="Activity" title="Connexions TCP/UDP" desc="Toutes les connexions actives et en écoute" color="indigo" />
      <NCollapse
        :title="`Connexions actives — ${filteredConns.length}/${connections.length}`"
        storageKey="diag-net-connections"
        defaultOpen
      >
        <template #header-extra>
          <div style="display:flex;gap:4px;flex-wrap:wrap">
            <NBadge v-for="s in ['all','Established','Listen','TimeWait','CloseWait']" :key="s"
              :variant="connFilter === s ? 'info' : 'default'"
              style="cursor:pointer" @click.stop="connFilter = s">
              {{ s === 'all' ? 'Tous' : s }}
            </NBadge>
          </div>
        </template>
        <div class="diag-search" style="margin-bottom:10px">
          <Search :size="14" />
          <input v-model="connSearch" placeholder="Filtrer par processus, IP..." class="diag-search-input" />
        </div>
        <div v-if="!filteredConns.length" class="diag-empty">Aucune connexion trouvée</div>
        <div class="table-wrap">
          <table class="data-table">
            <thead>
              <tr>
                <th>Proto</th><th>Processus</th><th>Adresse locale</th><th>Port local</th>
                <th>Adresse distante</th><th>Port distant</th><th>État</th><th>PID</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(c, i) in filteredConns.slice(0, 300)" :key="i">
                <td><NBadge :variant="c.protocol === 'TCP' ? 'info' : 'default'" style="font-size:10px">{{ c.protocol }}</NBadge></td>
                <td style="font-weight:500;max-width:120px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">{{ c.process_name || "—" }}</td>
                <td><code>{{ c.local_address || "*" }}</code></td>
                <td><code>{{ c.local_port }}</code></td>
                <td><code style="color:var(--text-secondary)">{{ c.remote_address || "*" }}</code></td>
                <td><code style="color:var(--text-secondary)">{{ c.remote_port || "*" }}</code></td>
                <td><NBadge :variant="stateVariant(c.state)" style="font-size:10px">{{ c.state }}</NBadge></td>
                <td class="muted">{{ c.pid }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </NCollapse>
    </div>
  </template>
</template>
