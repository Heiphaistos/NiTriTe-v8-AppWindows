/// <reference types="vite/client" />

// Variable injectée par Vite depuis package.json — source de vérité unique
declare const __APP_VERSION__: string;

interface Window {
  __hideNativeBoot?: () => void;
  __nitrite_sdi_active?: boolean;
}

// Shim pour que tsc puisse résoudre les imports *.vue sans vue-tsc
declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent
  export default component
}
