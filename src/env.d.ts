/// <reference types="vite/client" />

// Variable injectée par Vite depuis package.json — source de vérité unique
declare const __APP_VERSION__: string;

interface Window {
  __hideNativeBoot?: () => void;
  __nitrite_sdi_active?: boolean;
}
