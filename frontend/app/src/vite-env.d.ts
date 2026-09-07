/// <reference types="vite/client" />

declare const __APP_VERSION__: string
declare const __APP_BUILD_DATE__: string

interface ImportMetaEnv {
  readonly VITE_ROOMS_V2_ENABLED?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

