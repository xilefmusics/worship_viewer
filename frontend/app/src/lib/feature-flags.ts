export type WorshipRuntimeConfig = {
  roomsV2Enabled?: boolean
}

declare global {
  // eslint-disable-next-line no-var -- attach runtime config from /runtime-config.js
  var __WORSHIP_RUNTIME__: WorshipRuntimeConfig | undefined
}

export function isRoomsV2Enabled(): boolean {
  const runtime = globalThis.__WORSHIP_RUNTIME__
  if (typeof runtime?.roomsV2Enabled === 'boolean') {
    return runtime.roomsV2Enabled
  }
  return import.meta.env.VITE_ROOMS_V2_ENABLED === 'true'
}
