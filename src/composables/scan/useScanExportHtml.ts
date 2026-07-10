import { invoke, invokeRaw, useNotificationStore, fullRegPath, type Solution } from "./scanExportHelpers";
import type { ScanResult, BatteryDetailed } from "@/types/diagnostic";

export async function exportScanHtml(
  scanResult: ScanResult | null,
  scanProblems: string[],
  batteries: BatteryDetailed[],
  scanSolutions: Solution[]
) {
  if (!scanResult) return;
  const sr = scanResult;
  const now = new Date().toLocaleString();
  const h = (s: unknown) => String(s ?? "").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;");
  const ok  = (v: boolean, t="Activé", f="DÉSACTIVÉ") => `<span class="${v?'ok':'bad'}">${v?t:f}</span>`;
  const badge = (txt: string, cls: string) => `<span class="badge badge-${cls}">${h(txt)}</span>`;
  const sec = (title: string, icon: string, id = "") =>
    `<h2${id ? ` id="${id}"` : ""}>${icon} ${h(title)}</h2>`;
  const tbl = (head: string[], rows: string[][]) =>
    `<table><thead><tr>${head.map(c=>`<th>${h(c)}</th>`).join("")}</tr></thead><tbody>`
    + rows.map(r=>`<tr>${r.map(c=>`<td>${c}</td>`).join("")}</tr>`).join("")
    + `</tbody></table>`;

  const css = `
*{box-sizing:border-box;margin:0;padding:0}
html{scroll-behavior:smooth}
body{font-family:'Segoe UI',sans-serif;background:#0d0d0f;color:#e0e0e8;display:flex;min-height:100vh}
h1{font-size:22px;color:#f97316;margin-bottom:4px}
.subtitle{color:#888;font-size:12px;margin-bottom:28px}
h2{font-size:12px;text-transform:uppercase;letter-spacing:.1em;color:#f97316;border-bottom:1px solid #2e2e33;padding-bottom:5px;margin:28px 0 10px;scroll-margin-top:16px}
.part-title{font-size:15px;font-weight:700;color:#fff;margin:32px 0 4px;padding:10px 16px;background:linear-gradient(90deg,rgba(249,115,22,.2),transparent);border-left:4px solid #f97316;border-radius:0 6px 6px 0;scroll-margin-top:12px}
table{width:100%;border-collapse:collapse;font-size:12px;margin-bottom:14px}
th{background:#1a1a20;color:#aaa;text-align:left;padding:7px 12px;font-size:11px;text-transform:uppercase;letter-spacing:.06em}
td{padding:6px 12px;border-bottom:1px solid #1e1e24;vertical-align:top}
tr:hover td{background:#14141a}
code{font-family:monospace;background:#1a1a24;padding:1px 5px;border-radius:3px;font-size:11px;color:#e0e0e8;word-break:break-all}
pre{font-family:monospace;background:#111116;padding:8px 12px;border-radius:4px;font-size:10px;color:#9090a0;white-space:pre-wrap;word-break:break-all;max-height:180px;overflow-y:auto;border:1px solid #2a2a32;margin-top:4px}
.ok{color:#22c55e}.warn{color:#f59e0b}.bad{color:#ef4444}.muted{color:#666}
.badge{display:inline-block;padding:2px 7px;border-radius:10px;font-size:10px;font-weight:600}
.badge-success{background:rgba(34,197,94,.15);color:#22c55e}
.badge-warning{background:rgba(245,158,11,.15);color:#f59e0b}
.badge-danger{background:rgba(239,68,68,.15);color:#ef4444}
.badge-info{background:rgba(59,130,246,.15);color:#60a5fa}
.badge-neutral{background:rgba(156,163,175,.12);color:#9ca3af}
ul.problems{list-style:none;margin-bottom:14px}
ul.problems li{padding:6px 10px;border-left:3px solid #f59e0b;background:#1a1510;margin-bottom:4px;border-radius:0 4px 4px 0;font-size:12px}
.sev-critical td:first-child{border-left:3px solid #ef4444}
.sev-warning td:first-child{border-left:3px solid #f59e0b}
.sev-info td:first-child{border-left:3px solid #60a5fa}
.wmi-block{background:rgba(239,68,68,.07);border:1px solid rgba(239,68,68,.3);border-radius:6px;padding:10px 12px;margin-top:8px}
footer{margin-top:48px;color:#444;font-size:11px;text-align:center;border-top:1px solid #1e1e24;padding-top:16px}
/* ── Sidebar ── */
.sidebar{width:210px;min-width:210px;position:sticky;top:0;height:100vh;overflow-y:auto;background:#080810;border-right:1px solid #1a1a24;padding:0;flex-shrink:0;scrollbar-width:thin;scrollbar-color:#2a2a3a transparent}
.sidebar::-webkit-scrollbar{width:4px}.sidebar::-webkit-scrollbar-thumb{background:#2a2a3a;border-radius:4px}
.sb-header{padding:16px 14px 12px;border-bottom:1px solid #1a1a24}
.sb-logo{font-size:14px;font-weight:700;color:#f97316;letter-spacing:.02em}
.sb-date{font-size:10px;color:#475569;margin-top:2px}
.sb-group{font-size:9px;text-transform:uppercase;letter-spacing:.12em;color:#374151;padding:14px 14px 4px;font-weight:600}
.sidebar a{display:block;padding:5px 14px 5px 18px;color:#64748b;text-decoration:none;font-size:11px;border-left:2px solid transparent;transition:all .15s;line-height:1.4}
.sidebar a:hover{color:#e2e8f0;background:rgba(249,115,22,.06);border-left-color:#f97316}
.sidebar a.sub{padding-left:28px;font-size:10.5px;color:#4b5563}
.sidebar a.sub:hover{color:#cbd5e1;border-left-color:#f97316}
/* ── Content ── */
.content{flex:1;padding:32px 36px;max-width:1000px;overflow-x:hidden}`;

  const diskBar = (pct: number) => {
    const color = pct>90?'#ef4444':pct>80?'#f59e0b':'#22c55e';
    return `<div style="display:flex;align-items:center;gap:6px"><div style="flex:1;height:6px;background:#1e1e24;border-radius:3px"><div style="width:${pct.toFixed(0)}%;height:6px;background:${color};border-radius:3px"></div></div><span style="font-size:11px;min-width:34px;text-align:right">${pct.toFixed(0)}%</span></div>`;
  };

  const p1 = `
<div class="part-title" id="s1">🖥️ Partie 1 — Composants du PC</div>
${sec("Identité Système & BIOS","🔧","s1-bios")}
${tbl(["Propriété","Valeur"],[
  [h("Fabricant"),          h(sr.system_manufacturer||"N/A")],
  [h("Modèle"),             h(sr.system_model||"N/A")],
  [h("N° Série"),           `<code>${h(sr.system_serial||"N/A")}</code>`],
  [h("BIOS Fabricant"),     h(sr.bios_manufacturer||"N/A")],
  [h("BIOS Version"),       `<code>${h(sr.bios_version||"N/A")}</code>`],
  [h("BIOS Date"),          h(sr.bios_date||"N/A")],
  [h("BIOS Santé"),         badge(sr.bios_ok?"OK":"Erreur détection", sr.bios_ok?"success":"warning")],
])}
${sec("Composants Matériels","🔩","s1-hw")}
${tbl(["Composant","Détail"],[
  [h("CPU"),    h(`${sr.cpu_name} — ${sr.cpu_cores} cœurs / ${sr.cpu_threads||"?"}T @ ${sr.cpu_frequency_ghz||"?"}GHz`)],
  ...(sr.cpu_socket ? [[h("Socket CPU"), h(sr.cpu_socket)]] : []),
  ...((sr.cpu_l3_mb ?? 0) > 0 ? [[h("Cache L3"), h(sr.cpu_l3_mb+" MB")]] : []),
  [h("CPU Utilisation"), badge(sr.cpu_usage_percent?.toFixed(1)+"%", sr.cpu_usage_percent>80?"danger":sr.cpu_usage_percent>50?"warning":"success")],
  [h("CPU Température"),  sr.cpu_temperature && sr.cpu_temperature!=="N/A" ? badge(sr.cpu_temperature, (()=>{ const t=parseFloat(sr.cpu_temperature); return !isNaN(t)&&t>80?"danger":!isNaN(t)&&t>65?"warning":"success"; })()) : "<span class='muted'>N/A</span>"],
  [h("RAM"),    badge(`${sr.ram_used_gb?.toFixed(1)} / ${sr.ram_total_gb?.toFixed(0)} GB (${sr.ram_usage_percent?.toFixed(0)}%)`, sr.ram_usage_percent>85?"danger":sr.ram_usage_percent>65?"warning":"success")],
  [h("Config RAM"), h(sr.ram_detail||"N/A")],
  ...(sr.ram_slots?.length ? sr.ram_slots.map((sl: string) => { const [k,...v]=sl.split(":"); return [h(k.trim()), h(v.join(":").trim())]; }) : []),
  [h("Mémoire virtuelle"), h(`${sr.virtual_memory_total_mb||"?"} MB total / ${sr.virtual_memory_available_mb||"?"} MB libre`)],
  ...(sr.all_gpus?.length
    ? sr.all_gpus.map(g => [h(`GPU ${g.is_integrated?"(intégré)":"(dédié)"}`), h(`${g.name} — VRAM: ${g.vram_mb>=1024?(g.vram_mb/1024).toFixed(1)+"GB":g.vram_mb+"MB"}`)])
    : [[h("GPU"), h(`${sr.gpu_name||"N/A"} — VRAM: ${sr.gpu_vram_mb>=1024?(sr.gpu_vram_mb/1024).toFixed(1)+"GB":sr.gpu_vram_mb+"MB"}`)]] ),
  [h("Carte mère"), h(sr.motherboard||"N/A")],
  [h("Écrans"),  h(sr.monitors_detail||sr.screen_resolution||"N/A")],
  [h("Plan d'alim."), h(sr.power_plan||"N/A")],
])}
${sr.storage_items?.length ? sec("Stockage Physique","💾","s1-storage") + tbl(["Modèle","Type","Interface","Taille","Santé","Heures"],
  (sr.storage_items||[]).map((s)=>[
    h(s.model||"—"), h(s.media_type), h(s.interface_type), h(s.size_gb+" GB"),
    badge(s.health, s.health?.toLowerCase().includes("health")||s.health==="Sain"?"success":"warning"),
    (s.power_on_hours ?? 0) > 0 ? h(s.power_on_hours! >= 8760 ? (s.power_on_hours!/8760).toFixed(1)+" ans" : s.power_on_hours+" h") : "<span class='muted'>—</span>",
  ])
) : ""}
${sec("Espace Disque (Volumes)","💽","s1-volumes")}
${tbl(["Lecteur","Utilisation","Libre","Total"],
  (sr.disk_usage||[]).map((d)=>[`<code>${h(d.drive)}</code>`, diskBar(d.used_percent), h(d.free_gb.toFixed(1)+" GB"), h(d.total_gb.toFixed(0)+" GB")])
)}
${sec("Réseau","🌐","s1-net")}
${tbl(["Indicateur","Valeur"],[
  [h("Connectivité"),         ok(sr.network_ok,"OK (8.8.8.8)","Hors ligne ⚠")],
  [h("Adaptateurs actifs"),   h(sr.network_adapters_summary||"N/A")],
  [h("Ports en écoute"),      sr.open_ports?.length ? `<code>${h(sr.open_ports.slice(0,30).join(", "))}${sr.open_ports.length>30?" …":""}</code>` : "<span class='ok'>Aucun</span>"],
])}
${batteries?.length ? sec("Batterie","🔋","s1-battery") + batteries.map(b => tbl(["Propriété","Valeur"],[
  [h("Nom"),               h(b.name||"N/A")],
  [h("Statut"),              h(b.status||"N/A")],
  [h("Charge"),              badge(b.estimated_charge_remaining!=null?b.estimated_charge_remaining+"%":"N/A", b.estimated_charge_remaining>50?"success":b.estimated_charge_remaining>20?"warning":"danger")],
  [h("Autonomie estimée"),   h(b.estimated_run_time||"N/A")],
  [h("Capacité originale"),  h(b.design_capacity!=null?b.design_capacity+" mWh":"N/A")],
  [h("Capacité actuelle"),   h(b.full_charge_capacity!=null?b.full_charge_capacity+" mWh":"N/A")],
  [h("Santé batterie"),      badge(b.battery_health_percent!=null?b.battery_health_percent.toFixed(0)+"%":"N/A", b.battery_health_percent>=80?"success":b.battery_health_percent>=50?"warning":"danger")],
  [h("Chimie"),              h(b.chemistry||"N/A")],
  [h("Cycles"),              h(b.cycle_count!=null?String(b.cycle_count):"N/A")],
])).join("") : ""}`;

  const bitlockerRows = (sr.bitlocker_volumes||[]).map((bv)=>{
    const prot = bv.protection_status==="On"||bv.protection_status==="1";
    return [h(bv.drive), badge(prot?"Protégé":"Non protégé",prot?"success":"warning"),
      bv.recovery_password?`<code style="font-size:10px">${h(bv.recovery_password)}</code>`:"<span class='muted'>—</span>"];
  });
  const wingetRows = (sr.winget_upgradable||[]).map((u)=>[
    h(u.name), `<code>${h(u.id)}</code>`,
    badge(u.current_version||"?","warning"),
    `<span class="ok">${h(u.available_version||"?")}</span>`,
  ]);

  const p2 = `
<div class="part-title" id="s2">🪟 Partie 2 — Informations Windows</div>
${sec("Système Windows","🪟","s2-system")}
${tbl(["Indicateur","Valeur"],[
  [h("Version OS"),           h(sr.windows_version||"N/A")],
  [h("Uptime"),               h(sr.uptime_hours>=24?(sr.uptime_hours/24).toFixed(1)+" jours":sr.uptime_hours?.toFixed(1)+" heures")],
  [h("Activation"),           badge(sr.windows_activation||"Inconnu", sr.windows_activation==="Activé"||sr.windows_activation==="Licencié"?"success":"danger")],
  [h("Type licence"),         badge(sr.license_type||"N/A","info")],
  [h("Dernier KB"),           badge(sr.last_update_days>=0?"il y a "+sr.last_update_days+" jours":"N/A", sr.last_update_days<0?"neutral":sr.last_update_days<=30?"success":sr.last_update_days<=60?"warning":"danger")],
  [h("Redémarrage requis"),   badge(sr.pending_reboot?"Oui ⚠":"Non", sr.pending_reboot?"warning":"success")],
  [h("MAJ en attente (cache)"), badge(String(sr.pending_updates_cached>=0?sr.pending_updates_cached:"N/A"), sr.pending_updates_cached>10?"danger":sr.pending_updates_cached>0?"warning":"success")],
  [h("Logiciels installés"),  h(String(sr.installed_software_count||0))],
  [h("Services"),             h(`${sr.services_running||0} actifs / ${sr.services_stopped||0} arrêtés`)],
  [h("Prog. démarrage"),      h(String(sr.startup_count||0))],
  [h("Fichiers TEMP"),        badge(sr.temp_folder_size_mb>=1024?(sr.temp_folder_size_mb/1024).toFixed(1)+"GB":sr.temp_folder_size_mb?.toFixed(0)+"MB", sr.temp_folder_size_mb>2048?"danger":sr.temp_folder_size_mb>512?"warning":"success")],
  [h("Dernier BSOD"),         h(sr.last_bsod||"Aucun BSOD récent")],
])}
${sec("Licences & Chiffrement","🔑","s2-licenses")}
${tbl(["Propriété","Valeur"],[
  [h("Clé Windows"),          sr.windows_product_key?`<code>${h(sr.windows_product_key)}</code>`:"<span class='muted'>Non disponible</span>"],
  [h(sr.office_name||"Office"), sr.office_product_key?`<code>${h(sr.office_product_key)}</code>`:"<span class='muted'>N/A</span>"],
  [h("Type activation Win."),   h(sr.activation_type||"N/A")],
  [h("Type activation Office"), h(sr.office_activation_type||"N/A")],
])}
${bitlockerRows.length ? `<h3 id="s2-bitlocker" style="font-size:11px;color:#aaa;margin:8px 0 4px;text-transform:uppercase;scroll-margin-top:12px">BitLocker</h3>`+tbl(["Volume","Protection","Clé de récupération"],bitlockerRows) : "<p class='muted' style='font-size:12px;margin:4px 0 12px'>BitLocker non configuré</p>"}
${sec("Intégrité Système (DISM / SFC)","🛡️","s2-integrity")}
${tbl(["Outil","Statut"],[
  [h("DISM"), badge(sr.dism_status||"N/A", sr.dism_status?.toLowerCase().includes("sain")?"success":"danger")],
  [h("SFC"),  badge(sr.sfc_status||"N/A",  sr.sfc_status?.toLowerCase().includes("intég")||sr.sfc_status?.toLowerCase().includes("integ")?"success":"danger")],
])}
${sr.sfc_details ? `<pre>${h(sr.sfc_details)}</pre>` : ""}
${sr.dism_details && !sr.dism_status?.toLowerCase().includes("sain") ? `<pre>${h(sr.dism_details)}</pre>` : ""}
${sec("Mises à Jour — Gestionnaires de Paquets","📦","s2-updates")}
${tbl(["Gestionnaire","Statut"],[
  [h("WinGet"),         wingetRows.length ? badge(`⚠ ${wingetRows.length} logiciel(s) à mettre à jour`,"warning") : badge("✅ À jour","success")],
  [h("Chocolatey"),     sr.choco_upgradable?.length  ? badge(`⚠ ${sr.choco_upgradable.length} paquet(s) à mettre à jour`,"warning")  : badge("✅ À jour","success")],
  [h("Scoop"),          sr.scoop_upgradable?.length  ? badge(`⚠ ${sr.scoop_upgradable.length} paquet(s) à mettre à jour`,"warning")  : badge("✅ À jour","success")],
  [h("Windows Update"), sr.windows_updates_pending?.length ? badge(`⚠ ${sr.windows_updates_pending.length} KB en attente`,"warning") : sr.pending_updates_cached>0 ? badge(`⚠ ${sr.pending_updates_cached} (cache)`,"warning") : badge("✅ À jour","success")],
])}
<h3 style="font-size:11px;color:#aaa;margin:12px 0 4px;text-transform:uppercase;letter-spacing:.06em">WinGet</h3>
${wingetRows.length ? tbl(["Nom","ID","Version actuelle","Disponible"], wingetRows) : "<p class='muted' style='font-size:12px;padding:4px 0 10px'>Aucune mise à jour disponible</p>"}
<h3 style="font-size:11px;color:#aaa;margin:12px 0 4px;text-transform:uppercase;letter-spacing:.06em">Chocolatey</h3>
${sr.choco_upgradable?.length ? `<ul style="margin:0 0 10px;padding-left:20px;font-size:12px">${sr.choco_upgradable.slice(0,20).map((u:string)=>`<li>${h(u)}</li>`).join("")}${sr.choco_upgradable.length>20?`<li class="muted">+${sr.choco_upgradable.length-20} autres</li>`:""}</ul>` : "<p class='muted' style='font-size:12px;padding:4px 0 10px'>Aucune mise à jour disponible</p>"}
<h3 style="font-size:11px;color:#aaa;margin:12px 0 4px;text-transform:uppercase;letter-spacing:.06em">Scoop</h3>
${sr.scoop_upgradable?.length ? `<ul style="margin:0 0 10px;padding-left:20px;font-size:12px">${sr.scoop_upgradable.slice(0,20).map((u:string)=>`<li>${h(u)}</li>`).join("")}${sr.scoop_upgradable.length>20?`<li class="muted">+${sr.scoop_upgradable.length-20} autres</li>`:""}</ul>` : "<p class='muted' style='font-size:12px;padding:4px 0 10px'>Aucune mise à jour disponible</p>"}
<h3 style="font-size:11px;color:#aaa;margin:12px 0 4px;text-transform:uppercase;letter-spacing:.06em">Windows Update</h3>
${sr.windows_updates_pending?.length ? `<ul style="margin:0 0 10px;padding-left:20px;font-size:12px">${sr.windows_updates_pending.slice(0,20).map((u:string)=>`<li>${h(u)}</li>`).join("")}${sr.windows_updates_pending.length>20?`<li class="muted">+${sr.windows_updates_pending.length-20} autres</li>`:""}</ul>` : "<p class='muted' style='font-size:12px;padding:4px 0 10px'>Aucune mise à jour en attente</p>"}
${(sr.top_cpu?.length||sr.top_ram?.length) ? sec("Top 5 Processus (snapshot)","📊","s2-procs") + `<div style="display:grid;grid-template-columns:1fr 1fr;gap:16px">`
  + (sr.top_cpu?.length ? tbl(["PID","Processus","CPU (s)"], sr.top_cpu.map((p)=>[h(String(p.pid)),h(p.name),`<code>${h(String(p.value))}</code>`])) : "")
  + (sr.top_ram?.length ? tbl(["PID","Processus","RAM (MB)"], sr.top_ram.map((p)=>[h(String(p.pid)),h(p.name),`<code>${h(String(p.value))}</code>`])) : "")
  + `</div>` : ""}`;

  const wmiBlock = sr.wmi_subscriptions>0 ? `
<div class="wmi-block" id="s3-wmi">
  <p style="color:#ef4444;font-weight:600;margin-bottom:8px">⚠ ${sr.wmi_subscriptions} abonnement(s) WMI suspects détectés</p>
  ${tbl(["Nom","Type WMI","Chemin"],[...(sr.wmi_subscription_details||[]).map((s)=>[h(s.name||"(sans nom)"),badge(s.consumer_type,"danger"),`<code style="font-size:10px">${h(s.path)}</code>`])])}
  <p style="font-size:11px;color:#aaa;margin-top:6px">Commande de suppression : <code>Get-WmiObject -Namespace root\\subscription -Class __EventConsumer | Remove-WmiObject</code></p>
</div>` : "";

  const p3 = `
<div class="part-title" id="s3">🚨 Partie 3 — Problèmes &amp; Sécurité</div>
<div id="s3-problems">
${scanProblems.length
  ? `${sec(`Problèmes Détectés (${scanProblems.length})`, "⚠")}<ul class="problems">${scanProblems.map(p=>`<li>⚠ ${h(p)}</li>`).join("")}</ul>`
  : `<p style="color:#22c55e;padding:10px 0;font-size:13px">✅ Aucun problème critique détecté</p>`}
</div>
${scanSolutions.length ? sec("Solutions Recommandées","💡","s3-solutions") + tbl(["Problème","Action","Sévérité"],
  scanSolutions.map(s=>[h(s.problem), h(s.action),
    badge(s.severity==="critical"?"🔴 Critique":s.severity==="warning"?"🟡 Attention":"🔵 Info",
          s.severity==="critical"?"danger":s.severity==="warning"?"warning":"info")])
) : ""}
${sec("Sécurité","🔒","s3-security")}
${tbl(["Indicateur","Valeur"],[
  [h("Pare-feu Windows"),     ok(sr.firewall_enabled)],
  [h("Windows Defender"),     ok(sr.defender_enabled,"Actif","INACTIF ⚠")],
  [h("Antivirus tiers"),      h(sr.antivirus_installed||"Aucun (Defender)")],
  [h("Defs Defender"),        badge(sr.defender_definition_age_days>=0?sr.defender_definition_age_days+" j":"N/A", sr.defender_definition_age_days<0?"neutral":sr.defender_definition_age_days<=3?"success":sr.defender_definition_age_days<=7?"warning":"danger")],
  [h("Connectivité Internet"), ok(sr.network_ok,"OK","Hors ligne ⚠")],
  [h("Dernier BSOD"),         h(sr.last_bsod||"Aucun")],
])}
${sec("Sécurité Avancée","🛡️","s3-advanced")}
${tbl(["Indicateur","Valeur"],[
  [h("TPM"),          badge(sr.tpm_present?(sr.tpm_enabled?`Présent & Actif (v${sr.tpm_version||"?"})`:"Présent (désactivé)"):"Absent", sr.tpm_present&&sr.tpm_enabled?"success":"warning")],
  [h("Secure Boot"),  badge(sr.secure_boot?"Activé":"Désactivé", sr.secure_boot?"success":"warning")],
  [h("Niveau UAC"),   h(sr.uac_level||"Inconnu")],
  [h("RDP"),          badge(sr.rdp_enabled?"Activé ⚠":"Désactivé", sr.rdp_enabled?"warning":"success")],
  [h("SMBv1"),        badge(sr.smbv1_enabled?"Activé ⚠":"Désactivé", sr.smbv1_enabled?"danger":"success")],
  [h("Abonnements WMI"), badge(String(sr.wmi_subscriptions??0), sr.wmi_subscriptions>0?"danger":"success")],
  [h("Compte Invité"), badge(sr.guest_enabled?"Activé ⚠":"Désactivé", sr.guest_enabled?"warning":"success")],
  [h("Administrateurs locaux"), h(sr.local_admins?.join(", ")||"N/A")],
  [h("Dernier point de restauration"), badge(sr.last_restore_point||"N/A", sr.last_restore_point?.includes("Aucun")?"warning":"success")],
])}
${wmiBlock}
${(sr.suspicious_processes||[]).length ? sec(`Processus Hors Chemins Sécurisés (${sr.suspicious_processes.length})`,"🔍","s3-procs") + tbl(["Nom","PID","Raison","Chemin"],(sr.suspicious_processes||[]).map((p)=>[h(p.name),h(String(p.pid)),badge(p.reason,"warning"),`<code style="font-size:10px">${h(p.path)}</code>`])) : ""}
${(sr.suspicious_services||[]).length ? sec(`Services Tiers Actifs (${sr.suspicious_services.length})`,"⚙️","s3-services") + tbl(["Nom","Affichage","État","Chemin"],(sr.suspicious_services||[]).slice(0,20).map((s)=>[h(s.name),h(s.display_name),badge(s.state,"info"),`<code style="font-size:10px">${h(s.path)}</code>`])) : ""}
${(sr.autorun_entries||[]).length ? sec(`Entrées Autorun Tiers (${sr.autorun_entries.length})`,"🚀","s3-autoruns") + tbl(["Nom","Clé Registre","Exécutable"],(sr.autorun_entries||[]).slice(0,30).map((a)=>[h(a.name),`<code style="font-size:10px">${h(fullRegPath(a.location, a.name))}</code>`,`<code style="font-size:10px">${h(a.path)}</code>`])) : ""}
${(sr.susp_tasks||[]).length ? sec(`Tâches Planifiées Suspectes (${sr.susp_tasks_count})`,"📅","s3-tasks") + tbl(["Tâche","Chemin","Exécutable"],(sr.susp_tasks||[]).map((t)=>[h(t.name),badge(t.path,"neutral"),`<code style="font-size:10px">${h(t.exec)}</code>`])) : ""}
${(sr.recent_errors||[]).length ? sec(`Erreurs Récentes (${sr.recent_errors.length} — 48h)`,"🔴","s3-errors") + tbl(["Heure","Niveau","Source","Message"],(sr.recent_errors||[]).slice(0,20).map((e)=>[`<code>${h(e.time)}</code>`,badge(e.level, e.level?.toLowerCase().includes("crit")?"danger":"warning"),h(e.source),`<span style="font-size:11px">${h((e.message||"").substring(0,120))}</span>`])) : ""}
${(sr.scan_errors||[]).length ? sec(`Erreurs de Scan (${sr.scan_errors.length})`,"⚙️","s3-scan-errors") + `<ul style="margin:0 0 14px;padding-left:20px;font-size:12px">${sr.scan_errors.map((e:string)=>`<li><code style="color:#f59e0b">${h(e)}</code></li>`).join("")}</ul>` : ""}`;


  const scoreColor = scanProblems.length === 0 ? "#22c55e" : scanProblems.length <= 3 ? "#f59e0b" : "#ef4444";
  const scoreLabel = scanProblems.length === 0 ? "Aucun problème" : `${scanProblems.length} problème(s)`;

  const sidebar = `
<nav class="sidebar">
  <div class="sb-header">
    <div class="sb-logo">NiTriTe</div>
    <div class="sb-date">${now}</div>
    <div style="margin-top:8px;display:flex;align-items:center;gap:6px">
      <span style="width:8px;height:8px;border-radius:50%;background:${scoreColor};display:inline-block;flex-shrink:0"></span>
      <span style="font-size:10px;color:${scoreColor};font-weight:600">${scoreLabel}</span>
    </div>
  </div>
  <div class="sb-group">Composants PC</div>
  <a href="#s1">🖥️ Vue d'ensemble</a>
  <a href="#s1-bios" class="sub">🔧 Identité &amp; BIOS</a>
  <a href="#s1-hw" class="sub">🔩 Matériel</a>
  ${sr.storage_items?.length ? `<a href="#s1-storage" class="sub">💾 Stockage</a>` : ""}
  <a href="#s1-volumes" class="sub">💽 Espace disque</a>
  <a href="#s1-net" class="sub">🌐 Réseau</a>
  ${batteries?.length ? `<a href="#s1-battery" class="sub">🔋 Batterie</a>` : ""}
  <div class="sb-group">Windows</div>
  <a href="#s2">🪟 Vue d'ensemble</a>
  <a href="#s2-system" class="sub">🪟 Système</a>
  <a href="#s2-licenses" class="sub">🔑 Licences</a>
  ${bitlockerRows.length ? `<a href="#s2-bitlocker" class="sub">🔐 BitLocker</a>` : ""}
  <a href="#s2-integrity" class="sub">🛡️ DISM / SFC</a>
  <a href="#s2-updates" class="sub">📦 Mises à jour paquets</a>
  ${(sr.top_cpu?.length||sr.top_ram?.length) ? `<a href="#s2-procs" class="sub">📊 Top processus</a>` : ""}
  <div class="sb-group">Sécurité</div>
  <a href="#s3">🚨 Bilan sécurité</a>
  <a href="#s3-problems" class="sub">⚠ Problèmes${scanProblems.length ? ` (${scanProblems.length})` : ""}</a>
  ${scanSolutions.length ? `<a href="#s3-solutions" class="sub">💡 Solutions</a>` : ""}
  <a href="#s3-security" class="sub">🔒 Sécurité</a>
  <a href="#s3-advanced" class="sub">🛡️ Sec. avancée</a>
  ${sr.wmi_subscriptions>0 ? `<a href="#s3-wmi" class="sub" style="color:#ef4444">⚡ WMI (${sr.wmi_subscriptions})</a>` : ""}
  ${(sr.suspicious_processes||[]).length ? `<a href="#s3-procs" class="sub">🔍 Proc. suspects (${sr.suspicious_processes.length})</a>` : ""}
  ${(sr.suspicious_services||[]).length ? `<a href="#s3-services" class="sub">⚙️ Services tiers</a>` : ""}
  ${(sr.autorun_entries||[]).length ? `<a href="#s3-autoruns" class="sub">🚀 Autoruns</a>` : ""}
  ${(sr.susp_tasks||[]).length ? `<a href="#s3-tasks" class="sub">📅 Tâches</a>` : ""}
  ${(sr.recent_errors||[]).length ? `<a href="#s3-errors" class="sub">🔴 Erreurs (${sr.recent_errors.length})</a>` : ""}
  ${(sr.scan_errors||[]).length ? `<a href="#s3-scan-errors" class="sub" style="color:#f59e0b">⚙️ Erreurs scan (${sr.scan_errors.length})</a>` : ""}
</nav>`;

  const html = `<!DOCTYPE html>
<html lang="fr">
<head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Rapport Scan Complet — NiTriTe</title>
<style>${css}</style>
</head>
<body>
${sidebar}
<main class="content">
<h1>🔍 Rapport Scan Complet — NiTriTe</h1>
<p class="subtitle">Généré le ${now}</p>
${p1}${p2}${p3}
<footer>Rapport complet généré par <strong>NiTriTe v6.0.0</strong> — ${now}</footer>
</main>
</body></html>`;

  try {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const filePath = await save({ defaultPath: "scan_complet.html", filters: [{ name: "HTML", extensions: ["html"] }] });
    if (!filePath) return;
    await invoke("save_content_to_path", { path: filePath, content: html });
    useNotificationStore().success("Scan exporté (HTML)", filePath);
    await invokeRaw("open_path", { path: filePath }).catch(() => {});
  } catch (e: unknown) { useNotificationStore().error("Erreur export", String(e)); }
}
