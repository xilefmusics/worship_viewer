export const YOUTUBE_IFRAME_API_SRC = 'https://www.youtube.com/iframe_api'
export const YOUTUBE_NOCOOKIE_HOST = 'https://www.youtube-nocookie.com'

export const YOUTUBE_PLAYER_ENDED = 0
export const YOUTUBE_PLAYER_PLAYING = 1
export const YOUTUBE_PLAYER_PAUSED = 2
export const YOUTUBE_PLAYER_BUFFERING = 3
export const YOUTUBE_PLAYER_CUED = 5

export type YoutubeIframePlayer = {
  playVideo: () => void
  pauseVideo: () => void
  seekTo: (seconds: number, allowSeekAhead: boolean) => void
  setVolume: (volume: number) => void
  mute: () => void
  unMute: () => void
  getCurrentTime: () => number
  getDuration: () => number
  getPlayerState: () => number
  destroy: () => void
}

export type YoutubeIframePlayerEvent = {
  data: number
  target: YoutubeIframePlayer
}

export type YoutubeIframePlayerOptions = {
  videoId: string
  host?: string
  width?: string | number
  height?: string | number
  playerVars?: Record<string, string | number>
  events?: {
    onReady?: (event: YoutubeIframePlayerEvent) => void
    onStateChange?: (event: YoutubeIframePlayerEvent) => void
    onError?: (event: YoutubeIframePlayerEvent) => void
  }
}

export type YoutubeIframePlayerCtor = new (
  element: HTMLElement | string,
  options: YoutubeIframePlayerOptions,
) => YoutubeIframePlayer

type YoutubeWindow = Window & {
  YT?: { Player?: YoutubeIframePlayerCtor }
  onYouTubeIframeAPIReady?: () => void
}

const SCRIPT_ID = 'wv-youtube-iframe-api'
let loadPromise: Promise<YoutubeIframePlayerCtor> | null = null

export function resetYoutubeIframeApiForTests(): void {
  loadPromise = null
}

export function loadYoutubeIframeApi(): Promise<YoutubeIframePlayerCtor> {
  const win = window as YoutubeWindow
  if (win.YT?.Player) return Promise.resolve(win.YT.Player)
  if (loadPromise) return loadPromise

  loadPromise = new Promise((resolve, reject) => {
    const previousReady = win.onYouTubeIframeAPIReady
    const finish = (ctor: YoutubeIframePlayerCtor | undefined) => {
      if (ctor) {
        resolve(ctor)
        return
      }
      loadPromise = null
      reject(new Error('provider_unavailable'))
    }

    win.onYouTubeIframeAPIReady = () => {
      previousReady?.()
      finish(win.YT?.Player)
    }

    if (document.getElementById(SCRIPT_ID)) return

    const script = document.createElement('script')
    script.id = SCRIPT_ID
    script.src = YOUTUBE_IFRAME_API_SRC
    script.async = true
    script.onerror = () => {
      loadPromise = null
      reject(new Error('provider_unavailable'))
    }
    document.head.appendChild(script)
  })

  return loadPromise
}
