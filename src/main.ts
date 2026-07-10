import { createApp } from "vue";
import { createPinia } from "pinia";
import piniaPluginPersistedstate from "pinia-plugin-persistedstate";
import App from "./App.vue";
import router from "./router";
import "./assets/styles/main.css";
import "./assets/styles/tab-styles.css";
import "./assets/diagnostic.css";
import { logger, setupGlobalErrorHandlers } from "./utils/logger";

// ── Logger : intercepteurs globaux (window.onerror, unhandledrejection, console) ──
setupGlobalErrorHandlers();

const pinia = createPinia();
pinia.use(piniaPluginPersistedstate);

const app = createApp(App);

// Capture globale des erreurs Vue → logger + console
app.config.errorHandler = (err, _instance, info) => {
  console.error(`[Nitrite][Vue error][${info}]`, err);
  logger.vue(info, err);
};

app.use(pinia);
app.use(router);

app.mount("#app");
