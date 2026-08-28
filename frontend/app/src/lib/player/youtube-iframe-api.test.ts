/** @vitest-environment jsdom */

import { afterEach, describe, expect, it } from 'vitest'

import {
  YOUTUBE_IFRAME_API_SRC,
  loadYoutubeIframeApi,
  resetYoutubeIframeApiForTests,
} from '@/lib/player/youtube-iframe-api'

describe('loadYoutubeIframeApi', () => {
  afterEach(() => {
    resetYoutubeIframeApiForTests()
    document.getElementById('wv-youtube-iframe-api')?.remove()
    delete (window as { YT?: unknown }).YT
    delete (window as { onYouTubeIframeAPIReady?: unknown }).onYouTubeIframeAPIReady
  })

  it('rejects when the provider script fails to load', async () => {
    const pending = loadYoutubeIframeApi()
    const script = document.getElementById('wv-youtube-iframe-api') as HTMLScriptElement
    expect(script.src).toContain(YOUTUBE_IFRAME_API_SRC)
    script.dispatchEvent(new Event('error'))
    await expect(pending).rejects.toThrow('provider_unavailable')
  })

  it('resolves from onYouTubeIframeAPIReady and reuses the same Player', async () => {
    const Player = class {}
    const pending = loadYoutubeIframeApi()
    ;(window as { YT?: { Player: unknown } }).YT = { Player }
    ;(window as { YT?: { Player: unknown }; onYouTubeIframeAPIReady?: () => void }).onYouTubeIframeAPIReady?.()
    await expect(pending).resolves.toBe(Player)
    await expect(loadYoutubeIframeApi()).resolves.toBe(Player)
  })
})
