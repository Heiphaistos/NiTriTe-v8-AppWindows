import { defineStore } from "pinia";
import { ref, watch } from "vue";
import { invoke } from "@/utils/invoke";
import type { AppConfig } from "@/types/diagnostic";

// Température LLM valide : [0, 2]. Une valeur corrompue (parseFloat → NaN) ou une
// saisie vidée deviendrait NaN, sérialisée en "NaN"/null et propagée à la config.
function sanitizeTemp(n: number): number {
  return Number.isFinite(n) && n >= 0 && n <= 2 ? n : 0.7;
}

export const useAiStore = defineStore("ai", () => {
  const ollamaUrl   = ref(localStorage.getItem("ai_url")   ?? "http://localhost:11434");
  const ollamaModel = ref(localStorage.getItem("ai_model") ?? "llama3:8b");
  const temperature = ref(sanitizeTemp(parseFloat(localStorage.getItem("ai_temperature") ?? "0.7")));
  const isLoaded    = ref(false);

  // Persistance immédiate en localStorage (valeur assainie, jamais "NaN")
  watch(ollamaUrl,   v => localStorage.setItem("ai_url",   v));
  watch(ollamaModel, v => localStorage.setItem("ai_model", v));
  watch(temperature, v => localStorage.setItem("ai_temperature", String(sanitizeTemp(v))));

  // Charge depuis la config Rust au démarrage
  async function loadFromConfig() {
    try {
      const cfg = await invoke<AppConfig>("get_config");
      if (cfg.ollama_url)         ollamaUrl.value   = cfg.ollama_url;
      if (cfg.ollama_model)       ollamaModel.value = cfg.ollama_model;
      // Vérifier le type, pas la véracité : une température de 0.0 (déterministe)
      // est valide mais falsy, donc `if (cfg.ollama_temperature)` l'ignorerait.
      if (typeof cfg.ollama_temperature === "number") temperature.value = sanitizeTemp(cfg.ollama_temperature);
      isLoaded.value = true;
    } catch { isLoaded.value = true; }
  }

  // Sauvegarde dans la config Rust (appelé depuis SettingsPage)
  async function saveToConfig(extraConfig: Record<string, unknown> = {}) {
    try {
      const currentCfg = await invoke<AppConfig>("get_config");
      await invoke("save_config", {
        config: {
          ...currentCfg,
          ollama_url:         ollamaUrl.value,
          ollama_model:       ollamaModel.value,
          ollama_temperature: sanitizeTemp(temperature.value),
          ...extraConfig,
        },
      });
    } catch { /* config save non critique */ }
  }

  return { ollamaUrl, ollamaModel, temperature, isLoaded, loadFromConfig, saveToConfig };
});
