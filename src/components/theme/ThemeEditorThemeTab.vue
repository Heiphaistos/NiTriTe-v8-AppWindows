<script setup lang="ts">
import { computed } from "vue";
import { useThemeEditorStore, PRESET_THEMES, THEME_VAR_GROUPS, PRESET_THEME_GROUPS } from "@/stores/themeEditor";
import { Sparkles, Check, Save, Trash2 } from "lucide-vue-next";

const store = useThemeEditorStore();

function handleColorChange(key: string, e: Event) {
  store.setVar(key, (e.target as HTMLInputElement).value);
}

function handleRadiusChange(key: string, e: Event) {
  store.setVar(key, `${(e.target as HTMLInputElement).value}px`);
}

function radiusValue(val: string): number {
  return parseInt(val) || 0;
}

function presetsMatch(presetVars: Record<string, string>, editingVars: Record<string, string>): boolean {
  const pKeys = Object.keys(presetVars);
  const eKeys = Object.keys(editingVars);
  if (pKeys.length !== eKeys.length) return false;
  return pKeys.every(k => presetVars[k]?.toLowerCase().trim() === editingVars[k]?.toLowerCase().trim());
}

const currentPresetId = computed(() =>
  PRESET_THEMES.find(p => presetsMatch(p.vars as Record<string, string>, store.editingVars))?.id ?? null
);

interface RadiusVar { key: string; label: string; type: 'radius'; min?: number; max?: number; }
</script>

<template>
  <!-- Preset gallery groupée -->
  <div class="te-section">
    <div class="te-section-title">
      <Sparkles :size="13" />
      Thèmes Prédéfinis
      <span class="te-count">{{ PRESET_THEMES.length }}</span>
    </div>
    <div class="te-presets-grid">
      <template v-for="group in PRESET_THEME_GROUPS" :key="group.label">
        <div class="te-preset-group-label">{{ group.label }}</div>
        <button
          v-for="p in PRESET_THEMES.filter(t => (group.ids as readonly string[]).includes(t.id))"
          :key="p.id"
          class="te-preset-btn"
          :class="{ active: currentPresetId === p.id }"
          @click="store.loadPreset(p.id)"
          :title="p.label"
        >
          <span class="te-preset-dot" :style="{ background: p.accent }"></span>
          <span class="te-preset-label">{{ p.label }}</span>
          <Check v-if="currentPresetId === p.id" :size="11" class="te-preset-check" />
        </button>
      </template>
    </div>
  </div>

  <!-- Variable groups -->
  <div v-for="group in THEME_VAR_GROUPS" :key="group.label" class="te-section">
    <div class="te-section-title">{{ group.label }}</div>

    <!-- Color vars -->
    <div class="te-vars-list" v-if="group.vars.some(v => v.type === 'color')">
      <div v-for="v in group.vars.filter(v => v.type === 'color')" :key="v.key" class="te-var-row">
        <label class="te-var-label">{{ v.label }}</label>
        <div class="te-var-controls">
          <input type="color" class="te-color-input"
            :value="store.editingVars[v.key] ?? '#000000'"
            @input="handleColorChange(v.key, $event)"
          />
          <input type="text" class="te-hex-input"
            :value="store.editingVars[v.key] ?? '#000000'"
            @change="handleColorChange(v.key, $event)"
            maxlength="7"
          />
        </div>
      </div>
    </div>

    <!-- Radius vars -->
    <div class="te-vars-list" v-if="group.vars.some(v => v.type === 'radius')">
      <div v-for="v in (group.vars.filter(v => v.type === 'radius') as RadiusVar[])" :key="v.key" class="te-var-row te-var-row--radius">
        <label class="te-var-label">{{ v.label }}</label>
        <div class="te-var-controls">
          <input type="range" class="te-range-input"
            :min="v.min ?? 0"
            :max="v.max ?? 32"
            :value="radiusValue(store.editingVars[v.key] ?? '8px')"
            @input="handleRadiusChange(v.key, $event)"
          />
          <span class="te-range-val">{{ store.editingVars[v.key] ?? '8px' }}</span>
        </div>
      </div>
    </div>
  </div>

  <!-- Saved themes -->
  <div class="te-section" v-if="store.savedThemes.length > 0">
    <div class="te-section-title">
      <Save :size="13" />
      Mes Thèmes
      <span class="te-count">{{ store.savedThemes.length }}</span>
    </div>
    <div class="te-saved-list">
      <div v-for="t in store.savedThemes" :key="t.id" class="te-saved-item">
        <button class="te-saved-load" @click="store.loadSavedTheme(t.id)">{{ t.name }}</button>
        <button class="te-saved-del" @click="store.deleteSavedTheme(t.id)" title="Supprimer">
          <Trash2 :size="12" />
        </button>
      </div>
    </div>
  </div>
</template>
