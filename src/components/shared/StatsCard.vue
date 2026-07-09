<script setup lang="ts">
import { computed, type Component } from "vue";

const props = defineProps<{
  title: string;
  subtitle?: string;
  value: string | number;
  icon: Component;
  color?: "accent" | "success" | "warning" | "danger" | "info";
  progress?: number;
  trend?: { value: number; label: string };
}>();

const colorVar = computed(() => {
  const map: Record<string, string> = {
    accent: "var(--accent-primary)",
    success: "var(--success)",
    warning: "var(--warning)",
    danger: "var(--danger)",
    info: "var(--info)",
  };
  return map[props.color ?? "accent"];
});

const colorMuted = computed(() => {
  const map: Record<string, string> = {
    accent: "var(--accent-muted)",
    success: "var(--success-muted)",
    warning: "var(--warning-muted)",
    danger: "var(--danger-muted)",
    info: "var(--info-muted)",
  };
  return map[props.color ?? "accent"];
});

const colorGradient = computed(() => {
  const map: Record<string, string> = {
    accent:  "linear-gradient(135deg,rgba(249,115,22,.28),rgba(249,115,22,.08))",
    success: "linear-gradient(135deg,rgba(34,197,94,.28),rgba(34,197,94,.08))",
    warning: "linear-gradient(135deg,rgba(234,179,8,.28),rgba(234,179,8,.08))",
    danger:  "linear-gradient(135deg,rgba(239,68,68,.28),rgba(239,68,68,.08))",
    info:    "linear-gradient(135deg,rgba(59,130,246,.28),rgba(59,130,246,.08))",
  };
  return map[props.color ?? "accent"];
});

const colorGlow = computed(() => {
  const map: Record<string, string> = {
    accent:  "0 4px 16px rgba(249,115,22,.35)",
    success: "0 4px 16px rgba(34,197,94,.35)",
    warning: "0 4px 16px rgba(234,179,8,.35)",
    danger:  "0 4px 16px rgba(239,68,68,.35)",
    info:    "0 4px 16px rgba(59,130,246,.35)",
  };
  return map[props.color ?? "accent"];
});

const progressColor = computed(() => {
  if (props.progress === undefined) return "";
  if (props.progress > 90) return "var(--danger)";
  if (props.progress > 75) return "var(--warning)";
  return colorVar.value;
});
</script>

<template>
  <div class="stats-card" :style="{ borderTop: `2px solid ${colorVar}` }">
    <div class="stats-header">
      <div class="stats-icon-wrap" :style="{ background: colorGradient, boxShadow: colorGlow }">
        <component :is="icon" :size="20" :style="{ color: colorVar }" />
      </div>
      <div class="stats-info">
        <div class="stats-title">{{ title }}</div>
        <div v-if="subtitle" class="stats-subtitle">{{ subtitle }}</div>
      </div>
    </div>
    <div class="stats-value">{{ value }}</div>
    <div v-if="progress !== undefined" class="stats-progress">
      <div class="stats-progress-track">
        <div class="stats-progress-bar" :style="{ width: `${Math.min(100, progress)}%`, background: progressColor }" />
      </div>
    </div>
    <div v-if="trend" class="stats-trend" :class="{ 'trend-up': trend.value > 0, 'trend-down': trend.value < 0 }">
      <span>{{ trend.value > 0 ? "+" : "" }}{{ trend.value }}%</span>
      <span class="trend-label">{{ trend.label }}</span>
    </div>
  </div>
</template>

<style scoped>
.stats-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  padding: 18px;
  transition: all var(--transition-normal);
  overflow: hidden;
}

.stats-card:hover {
  border-color: var(--border-hover);
  transform: translateY(-2px);
  box-shadow: var(--shadow-lg);
}

.stats-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 14px;
}

.stats-icon-wrap {
  width: 42px;
  height: 42px;
  border-radius: var(--radius-lg);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.stats-info { min-width: 0; }

.stats-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.stats-subtitle {
  font-size: 11px;
  color: var(--text-secondary);
  margin-top: 1px;
}

.stats-value {
  font-size: 30px;
  font-weight: 700;
  color: var(--text-primary);
  font-family: "JetBrains Mono", monospace;
  line-height: 1.2;
}

.stats-progress {
  margin-top: 12px;
}

.stats-progress-track {
  height: 5px;
  border-radius: 99px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  overflow: hidden;
}

.stats-progress-bar {
  height: 100%;
  border-radius: 99px;
  transition: width 500ms ease;
}

.stats-trend {
  margin-top: 8px;
  font-size: 11px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.trend-up { color: var(--success); }
.trend-down { color: var(--danger); }
.trend-label { color: var(--text-muted); }
</style>
