export function canPlayNativeHls(el: HTMLMediaElement): boolean {
  return el.canPlayType('application/vnd.apple.mpegurl') !== ''
}

export type HlsClientInstance = {
  loadSource: (url: string) => void
  attachMedia: (el: HTMLMediaElement) => void
  destroy: () => void
  on: (event: string, listener: (event: string, data: { fatal?: boolean }) => void) => void
}

export type HlsModule = {
  default: {
    new (): HlsClientInstance
    isSupported: () => boolean
    Events: { ERROR: string; MANIFEST_PARSED: string }
  }
}

export async function loadHlsModule(): Promise<HlsModule> {
  return import('hls.js') as Promise<HlsModule>
}
