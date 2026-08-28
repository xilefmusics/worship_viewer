import { act, render, screen, waitFor } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { AvProjectedYoutube } from '@/components/player/av/AvProjectedYoutube'
import { DEFAULT_AV_PREFERENCES } from '@/lib/player/av-preferences'
import { buildAvPlaybackIntent } from '@/lib/player/av-projection-playback'
import { buildAvProjectionCommand } from '@/lib/player/av-projection-protocol'
import {
  YOUTUBE_NOCOOKIE_HOST,
  YOUTUBE_PLAYER_ENDED,
  YOUTUBE_PLAYER_PLAYING,
  type YoutubeIframePlayer,
  type YoutubeIframePlayerCtor,
  type YoutubeIframePlayerOptions,
} from '@/lib/player/youtube-iframe-api'

const layers = {
  contentLayer: DEFAULT_AV_PREFERENCES.contentLayer,
  backgroundLayer: DEFAULT_AV_PREFERENCES.backgroundLayer,
  transition: DEFAULT_AV_PREFERENCES.transition,
}

function youtubeCommand(
  commandId: number,
  playback = buildAvPlaybackIntent({ action: 'play' }),
  screenState: 'live' | 'blank' | 'blackout' = 'live',
) {
  return buildAvProjectionCommand({
    sessionId: 'shared',
    commandId,
    ...layers,
    screenState,
    itemTitle: 'Clip',
    nextPreview: null,
    content: {
      type: 'youtube',
      videoId: 'dQw4w9WgXcQ',
      canonicalUrl: 'https://www.youtube.com/watch?v=dQw4w9WgXcQ',
    },
    playback,
  })
}

function mockApi() {
  const player: YoutubeIframePlayer = {
    playVideo: vi.fn(),
    pauseVideo: vi.fn(),
    seekTo: vi.fn(),
    setVolume: vi.fn(),
    mute: vi.fn(),
    unMute: vi.fn(),
    getCurrentTime: () => 1.5,
    getDuration: () => 12,
    getPlayerState: () => YOUTUBE_PLAYER_PLAYING,
    destroy: vi.fn(),
  }
  let options: YoutubeIframePlayerOptions | undefined
  const Player = function Player(_el: HTMLElement, next: YoutubeIframePlayerOptions) {
    options = next
    queueMicrotask(() => next.events?.onReady?.({ data: -1, target: player }))
    return player
  } as unknown as YoutubeIframePlayerCtor
  return {
    player,
    emit: (data: number) => options?.events?.onStateChange?.({ data, target: player }),
    emitError: (data: number) => options?.events?.onError?.({ data, target: player }),
    loadApi: async () => Player,
  }
}

describe('AvProjectedYoutube', () => {
  it('I5: loads the nocookie embed and plays through the iframe API', async () => {
    const api = mockApi()
    const onAck = vi.fn()
    render(<AvProjectedYoutube command={youtubeCommand(1)} onAck={onAck} loadApi={api.loadApi} />)
    await waitFor(() => expect(api.player.playVideo).toHaveBeenCalled())
    expect(api.player.setVolume).toHaveBeenCalledWith(100)
    act(() => api.emit(YOUTUBE_PLAYER_PLAYING))
    expect(onAck).toHaveBeenCalledWith(
      true,
      expect.objectContaining({ status: 'playing', durationMs: 12000 }),
      undefined,
    )
    expect(screen.getByTestId('av-projected-youtube')).toBeInTheDocument()
    expect(YOUTUBE_NOCOOKIE_HOST).toContain('youtube-nocookie.com')
  })

  it('I5: loops on ended instead of using a playlist, and ignores stale events after destroy', async () => {
    const api = mockApi()
    const onAck = vi.fn()
    const { unmount } = render(
      <AvProjectedYoutube
        command={youtubeCommand(1, buildAvPlaybackIntent({ action: 'play', loop: true }))}
        onAck={onAck}
        loadApi={api.loadApi}
      />,
    )
    await waitFor(() => expect(api.player.playVideo).toHaveBeenCalled())
    act(() => api.emit(YOUTUBE_PLAYER_ENDED))
    expect(api.player.seekTo).toHaveBeenCalledWith(0, true)
    expect(api.player.playVideo).toHaveBeenCalled()
    unmount()
    onAck.mockClear()
    act(() => api.emit(YOUTUBE_PLAYER_PLAYING))
    expect(onAck).not.toHaveBeenCalled()
  })

  it('I5: maps embedding-disabled provider errors without leaking payloads', async () => {
    const api = mockApi()
    const onAck = vi.fn()
    render(<AvProjectedYoutube command={youtubeCommand(1)} onAck={onAck} loadApi={api.loadApi} />)
    await waitFor(() => expect(api.player.playVideo).toHaveBeenCalled())
    act(() => api.emitError(150))
    expect(onAck).toHaveBeenCalledWith(
      false,
      expect.objectContaining({ status: 'error' }),
      expect.objectContaining({ code: 'embed_blocked' }),
    )
  })

  it('I5: rejects an invalid video id without loading the provider', async () => {
    const loadApi = vi.fn(async () => {
      throw new Error('should not load')
    })
    const onAck = vi.fn()
    render(
      <AvProjectedYoutube
        command={buildAvProjectionCommand({
          sessionId: 'shared',
          commandId: 1,
          ...layers,
          screenState: 'live',
          itemTitle: 'Bad',
          nextPreview: null,
          content: { type: 'youtube', videoId: 'not-valid', canonicalUrl: 'https://example.com' },
          playback: buildAvPlaybackIntent({ action: 'play' }),
        })}
        onAck={onAck}
        loadApi={loadApi}
      />,
    )
    await waitFor(() => expect(onAck).toHaveBeenCalled())
    expect(loadApi).not.toHaveBeenCalled()
    expect(onAck).toHaveBeenCalledWith(false, undefined, expect.objectContaining({ code: 'load_failed' }))
  })
})
