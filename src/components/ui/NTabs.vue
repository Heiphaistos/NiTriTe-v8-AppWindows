<script setup lang="ts">
import { ref, watch, type Component } from "vue";

const props = defineProps<{
  tabs: { id: string; label: string; icon?: Component }[];
  modelValue?: string;
  wrap?: boolean;
}>();

const emit = defineEmits<{ "update:modelValue": [id: string] }>();

const activeTab = ref(props.modelValue ?? props.tabs[0]?.id ?? "");

watch(() => props.modelValue, (v) => { if (v) activeTab.value = v; });

function selectTab(id: string) {
  activeTab.value = id;
  emit("update:modelValue", id);
}
</script>

<template>
  <div class="n-tabs">
    <div class="tabs-header" :class="{ 'tabs-header--wrap': wrap }">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="tab-btn"
        :class="{ active: activeTab === tab.id }"
        @click="selectTab(tab.id)"
      >
        <component v-if="tab.icon" :is="tab.icon" :size="14" />
        {{ tab.label }}
      </button>
    </div>
    <div class="tabs-content">
      <slot :active-tab="activeTab" />
    </div>
  </div>
</template>

<style scoped>
.tabs-header {
  display: flex;
  gap: 2px;
  border-bottom: 1px solid var(--border);
  padding: 0 4px;
}

.tabs-header--wrap {
  flex-wrap: wrap;
  border-bottom: none;
  gap: 4px 2px;
  padding-bottom: 4px;
}

.tabs-header--wrap .tab-btn {
  border: 1px solid var(--border);
  border-radius: 6px;
  margin-bottom: 0;
}

.tabs-header--wrap .tab-btn.active {
  border-color: var(--accent-primary);
  background: color-mix(in srgb, var(--accent-primary) 12%, transparent);
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border: none;
  background: none;
  color: var(--text-muted);
  font-family: inherit;
  font-size: 13px;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all var(--transition-fast);
  margin-bottom: -1px;
}

.tab-btn:hover {
  color: var(--text-primary);
}

.tab-btn.active {
  color: var(--accent-primary);
  border-bottom-color: var(--accent-primary);
}

.tabs-content {
  padding: 16px 0;
}
</style>
