export function isRoomsV2Enabled(): boolean {
  return import.meta.env.VITE_ROOMS_V2_ENABLED === 'true'
}
