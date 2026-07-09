<script setup lang="ts">
import { Cpu, MemoryStick, Monitor } from "lucide-vue-next";
import NBadge from "@/components/ui/NBadge.vue";
import type { ScanResult } from "@/types/diagnostic";

defineProps<{ scanResult: ScanResult }>();
</script>

<template>
  <div class="diag-section">
    <p class="diag-section-label" style="margin:0 0 8px 0">Composants matériels</p>
    <div class="info-grid">
      <!-- CPU -->
      <div class="info-row">
        <span style="display:flex;align-items:center;gap:4px"><Cpu :size="12" /> Processeur</span>
        <span>{{ scanResult.cpu_name }}</span>
      </div>
      <div class="info-row"><span>Cœurs / Threads</span>
        <span>{{ scanResult.cpu_cores }} cœurs / {{ scanResult.cpu_threads > 0 ? scanResult.cpu_threads : '—' }} threads</span>
      </div>
      <div class="info-row"><span>Fréquence max</span>
        <span>{{ scanResult.cpu_frequency_ghz > 0 ? scanResult.cpu_frequency_ghz + ' GHz' : '—' }}</span>
      </div>
      <div v-if="scanResult.cpu_socket" class="info-row"><span>Socket</span><span>{{ scanResult.cpu_socket }}</span></div>
      <div v-if="(scanResult.cpu_l3_mb ?? 0) > 0" class="info-row"><span>Cache L3</span><span>{{ scanResult.cpu_l3_mb }} MB</span></div>
      <div class="info-row"><span>Utilisation CPU</span>
        <NBadge :variant="scanResult.cpu_usage_percent > 80 ? 'danger' : scanResult.cpu_usage_percent > 50 ? 'warning' : 'success'">
          {{ scanResult.cpu_usage_percent.toFixed(1) }}%
        </NBadge>
      </div>
      <div v-if="scanResult.cpu_temperature && scanResult.cpu_temperature !== 'N/A'" class="info-row">
        <span>Température CPU</span>
        <NBadge :variant="parseInt(scanResult.cpu_temperature) > 80 ? 'danger' : parseInt(scanResult.cpu_temperature) > 65 ? 'warning' : 'success'">
          {{ scanResult.cpu_temperature }}
        </NBadge>
      </div>
      <!-- RAM -->
      <div class="info-row">
        <span style="display:flex;align-items:center;gap:4px"><MemoryStick :size="12" /> RAM utilisée</span>
        <NBadge :variant="scanResult.ram_usage_percent > 85 ? 'danger' : scanResult.ram_usage_percent > 65 ? 'warning' : 'success'">
          {{ scanResult.ram_used_gb.toFixed(1) }} / {{ scanResult.ram_total_gb.toFixed(0) }} GB ({{ scanResult.ram_usage_percent.toFixed(0) }}%)
        </NBadge>
      </div>
      <div v-if="scanResult.ram_detail" class="info-row"><span>Configuration RAM</span><span>{{ scanResult.ram_detail }}</span></div>
      <template v-if="scanResult.ram_slots?.length">
        <div v-for="(slot, si) in scanResult.ram_slots" :key="si" class="info-row">
          <span style="padding-left:10px;color:var(--text-muted);font-size:11px">{{ slot.split(':')[0] }}</span>
          <span style="font-size:11px">{{ slot.split(':').slice(1).join(':').trim() }}</span>
        </div>
      </template>
      <!-- GPU (iGPU + dGPU) -->
      <template v-if="scanResult.all_gpus?.length">
        <div v-for="(gpu, gi) in scanResult.all_gpus" :key="gi" class="info-row">
          <span style="display:flex;align-items:center;gap:4px">
            <Monitor :size="12" />
            {{ gpu.is_integrated ? 'GPU intégré' : 'GPU dédié' }}
          </span>
          <div style="display:flex;align-items:center;gap:6px;flex-wrap:wrap">
            <span>{{ gpu.name }}</span>
            <NBadge v-if="gpu.vram_mb > 0" variant="neutral" style="font-size:10px">
              {{ gpu.vram_mb >= 1024 ? (gpu.vram_mb/1024).toFixed(1)+' GB' : gpu.vram_mb+' MB' }} VRAM
            </NBadge>
          </div>
        </div>
      </template>
      <template v-else-if="scanResult.gpu_name">
        <div class="info-row">
          <span style="display:flex;align-items:center;gap:4px"><Monitor :size="12" /> GPU</span>
          <span>{{ scanResult.gpu_name }}</span>
        </div>
        <div v-if="scanResult.gpu_vram_mb > 0" class="info-row">
          <span>VRAM GPU</span>
          <span>{{ scanResult.gpu_vram_mb >= 1024 ? (scanResult.gpu_vram_mb/1024).toFixed(1)+' GB' : scanResult.gpu_vram_mb+' MB' }}</span>
        </div>
      </template>
      <!-- Carte mère -->
      <div v-if="scanResult.motherboard" class="info-row"><span>Carte mère</span><span>{{ scanResult.motherboard }}</span></div>
      <!-- Écrans -->
      <div v-if="scanResult.monitors_detail" class="info-row info-full">
        <span style="display:flex;align-items:center;gap:4px"><Monitor :size="12" /> Écran(s)</span>
        <span style="font-size:11px">{{ scanResult.monitors_detail }}</span>
      </div>
      <div v-else-if="scanResult.screen_resolution" class="info-row"><span>Résolution</span><span>{{ scanResult.screen_resolution }}</span></div>
      <!-- Réseau -->
      <div v-if="scanResult.network_adapters_summary" class="info-row info-full">
        <span>Interfaces réseau actives</span><span style="font-size:11px">{{ scanResult.network_adapters_summary }}</span>
      </div>
    </div>
  </div>
</template>
