<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@/utils/invoke";
import NButton from "@/components/ui/NButton.vue";
import { Wifi, Globe, RefreshCw, Terminal, CheckCircle, XCircle } from "lucide-vue-next";

interface RepairResult { success: boolean; output: string; command: string; }

const emit = defineEmits<{ (e: "result", r: RepairResult): void }>();

const pingTarget = ref("8.8.8.8");
const staticIp = ref("");
const staticMask = ref("255.255.255.0");
const staticGateway = ref("");
const staticDns = ref("8.8.8.8");
const selectedIface = ref("");
const output = ref("");
const isLoading = ref(false);
const lastSuccess = ref<boolean | null>(null);

async function run(cmd: string, label?: string) {
  isLoading.value = true;
  try {
    const r = await invoke<RepairResult>("winpe_run_command", { command: cmd });
    output.value = r.output;
    lastSuccess.value = r.success;
    emit("result", { ...r, command: label || cmd });
  } catch (e) { output.value = String(e); lastSuccess.value = false; }
  finally { isLoading.value = false; }
}

const quickCmds = [
  { label: "Config IP complète",    cmd: "ipconfig /all" },
  { label: "Interfaces réseau",      cmd: "netsh interface show interface" },
  { label: "Table de routage",       cmd: "route print" },
  { label: "Connexions actives",     cmd: "netstat -an" },
  { label: "DNS cache",              cmd: "ipconfig /displaydns" },
  { label: "Vider cache DNS",        cmd: "ipconfig /flushdns" },
  { label: "Renouveler IP (DHCP)",   cmd: "ipconfig /renew" },
  { label: "Libérer bail DHCP",      cmd: "ipconfig /release" },
  { label: "Reset Winsock",         cmd: "netsh winsock reset" },
  { label: "Reset TCP/IP stack",     cmd: "netsh int ip reset" },
  { label: "DNS Cloudflare",         cmd: "netsh interface ip set dns \"Ethernet\" static 1.1.1.1 primary" },
  { label: "DNS Google",             cmd: "netsh interface ip set dns \"Ethernet\" static 8.8.8.8 primary" },
  { label: "Lister profils WiFi",    cmd: "netsh wlan show profiles" },
  { label: "Adaptateurs réseau",     cmd: "Get-NetAdapter | Select-Object Name,Status,LinkSpeed | Format-Table" },
  { label: "Adresses IP actives",    cmd: "Get-NetIPAddress | Where-Object { $_.AddressFamily -eq 'IPv4' } | Select-Object InterfaceAlias,IPAddress,PrefixLength | Format-Table" },
];

function isValidNetTarget(v: string) { return /^[a-zA-Z0-9.\-:]+$/.test(v.trim()); }
function isValidIface(v: string) { return v.length > 0 && !v.includes('"') && !v.includes(';') && !v.includes('`'); }

async function ping() {
  const target = pingTarget.value.trim();
  if (!isValidNetTarget(target)) { output.value = "Cible invalide — seuls chiffres, lettres, points, tirets et ':' autorisés."; lastSuccess.value = false; return; }
  await run(`ping -n 4 ${target}`, `ping ${target}`);
}

async function setStaticIp() {
  if (!selectedIface.value || !staticIp.value) return;
  if (!isValidIface(selectedIface.value)) { output.value = "Nom d'interface invalide."; lastSuccess.value = false; return; }
  if (!isValidNetTarget(staticIp.value) || !isValidNetTarget(staticMask.value)) { output.value = "Adresse IP ou masque invalide."; lastSuccess.value = false; return; }
  if (staticGateway.value && !isValidNetTarget(staticGateway.value)) { output.value = "Passerelle invalide."; lastSuccess.value = false; return; }
  if (staticDns.value && !isValidNetTarget(staticDns.value)) { output.value = "DNS invalide."; lastSuccess.value = false; return; }
  const cmd = `netsh interface ip set address "${selectedIface.value}" static ${staticIp.value} ${staticMask.value} ${staticGateway.value || ""}`;
  await run(cmd, `Set IP statique ${staticIp.value}`);
  if (staticDns.value) {
    await run(`netsh interface ip set dns "${selectedIface.value}" static ${staticDns.value}`, "Set DNS");
  }
}

async function setDhcp() {
  if (!selectedIface.value) return;
  if (!isValidIface(selectedIface.value)) { output.value = "Nom d'interface invalide."; lastSuccess.value = false; return; }
  await run(`netsh interface ip set address "${selectedIface.value}" dhcp`, "Set DHCP");
  await run(`netsh interface ip set dns "${selectedIface.value}" dhcp`, "Set DNS DHCP");
}
</script>

<template>
  <div class="net-tab">
    <!-- Actions rapides -->
    <div class="section-card">
      <h3 class="section-title"><Terminal :size="15" /> Commandes Réseau Rapides</h3>
      <div class="cmd-grid">
        <button v-for="c in quickCmds" :key="c.label" class="cmd-btn" :disabled="isLoading" @click="run(c.cmd, c.label)">
          {{ c.label }}
        </button>
      </div>
    </div>

    <!-- Ping -->
    <div class="section-card">
      <h3 class="section-title"><Globe :size="15" /> Test de Connectivité</h3>
      <div class="row-form">
        <input v-model="pingTarget" class="form-input" placeholder="IP ou nom de domaine (ex: 8.8.8.8)" />
        <NButton variant="primary" size="sm" :loading="isLoading" @click="ping">
          <Wifi :size="13" /> Ping
        </NButton>
      </div>
    </div>

    <!-- IP statique -->
    <div class="section-card">
      <h3 class="section-title"><RefreshCw :size="15" /> Configuration IP Manuelle</h3>
      <div class="ip-grid">
        <div class="form-group">
          <label class="form-label">Interface</label>
          <input v-model="selectedIface" class="form-input" placeholder="Ex: Ethernet, Local Area Connection" />
        </div>
        <div class="form-group">
          <label class="form-label">Adresse IP</label>
          <input v-model="staticIp" class="form-input" placeholder="192.168.1.100" />
        </div>
        <div class="form-group">
          <label class="form-label">Masque de sous-réseau</label>
          <input v-model="staticMask" class="form-input" placeholder="255.255.255.0" />
        </div>
        <div class="form-group">
          <label class="form-label">Passerelle</label>
          <input v-model="staticGateway" class="form-input" placeholder="192.168.1.1" />
        </div>
        <div class="form-group">
          <label class="form-label">Serveur DNS</label>
          <input v-model="staticDns" class="form-input" placeholder="8.8.8.8" />
        </div>
      </div>
      <div class="tool-actions">
        <NButton variant="primary" size="sm" :disabled="!selectedIface || !staticIp || isLoading" @click="setStaticIp">Appliquer IP statique</NButton>
        <NButton variant="ghost" size="sm" :disabled="!selectedIface || isLoading" @click="setDhcp">Remettre en DHCP</NButton>
      </div>
    </div>

    <!-- Output -->
    <div v-if="output" class="output-panel" :class="lastSuccess === false ? 'error' : lastSuccess ? 'success' : ''">
      <div class="output-header">
        <CheckCircle v-if="lastSuccess" :size="13" style="color:var(--success)" />
        <XCircle v-else-if="lastSuccess === false" :size="13" style="color:var(--danger)" />
        <span>Résultat</span>
      </div>
      <pre class="output-pre">{{ output }}</pre>
    </div>
  </div>
</template>

<style scoped>
.net-tab { display: flex; flex-direction: column; gap: 14px; }
.section-card { background: var(--bg-secondary); border: 1px solid var(--border); border-radius: var(--radius-xl); padding: 14px; display: flex; flex-direction: column; gap: 12px; }
.section-title { display: flex; align-items: center; gap: 8px; font-size: 13px; font-weight: 700; color: var(--text-primary); }
.cmd-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 6px; }
.cmd-btn { padding: 6px 10px; background: var(--bg-primary); border: 1px solid var(--border); border-radius: var(--radius-md); font-size: 11px; color: var(--text-secondary); cursor: pointer; transition: all .15s; text-align: left; font-family: inherit; }
.cmd-btn:hover:not(:disabled) { border-color: var(--accent-primary); color: var(--accent-primary); }
.cmd-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.row-form { display: flex; gap: 8px; align-items: center; }
.form-input { background: var(--bg-primary); border: 1px solid var(--border); border-radius: var(--radius-md); padding: 6px 10px; font-size: 12px; color: var(--text-primary); flex: 1; font-family: monospace; }
.ip-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 10px; }
.form-group { display: flex; flex-direction: column; gap: 4px; }
.form-label { font-size: 11px; font-weight: 600; color: var(--text-muted); }
.tool-actions { display: flex; gap: 8px; flex-wrap: wrap; }
.output-panel { background: var(--bg-primary); border: 1px solid var(--border); border-radius: var(--radius-lg); padding: 12px; }
.output-panel.success { border-color: var(--success); }
.output-panel.error { border-color: var(--danger); }
.output-header { display: flex; align-items: center; gap: 6px; font-size: 11px; font-weight: 600; color: var(--text-muted); margin-bottom: 8px; }
.output-pre { font-size: 11px; font-family: "JetBrains Mono", monospace; color: var(--text-secondary); white-space: pre-wrap; word-break: break-all; max-height: 320px; overflow-y: auto; }
</style>
