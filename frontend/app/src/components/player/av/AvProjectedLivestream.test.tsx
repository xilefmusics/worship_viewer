import { render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { AvProjectedLivestream } from '@/components/player/av/AvProjectedLivestream'
import { DEFAULT_AV_PREFERENCES } from '@/lib/player/av-preferences'
import { buildAvPlaybackIntent } from '@/lib/player/av-projection-playback'
import { buildAvProjectionCommand } from '@/lib/player/av-projection-protocol'
import type { HlsModule } from '@/lib/player/av-hls-client'

const layers = {
  contentLayer: DEFAULT_AV_PREFERENCES.contentLayer,
  backgroundLayer: DEFAULT_AV_PREFERENCES.backgroundLayer,
  transition: DEFAULT_AV_PREFERENCES.transition,
}

function liveCommand(
  commandId: number,
  streamType: 'hls' | 'direct' = 'hls',
  playback = buildAvPlaybackIntent({ action: 'play' }),
) {
  return buildAvProjectionCommand({
    sessionId: 'shared',
    commandId,
    ...layers,
    screenState: 'live',
    itemTitle: 'Stream',
    nextPreview: null,
    content: { type: 'livestream', url: 'https://example.com/live.m3u8', streamType },
    playback,
  })
}

describe('AvProjectedLivestream', () => {
  beforeEach(() => {
    Object.defineProperty(HTMLMediaElement.prototype, 'paused', {
      configurable: true,
      get() {
        return (this as HTMLMediaElement & { _paused?: boolean })._paused !== false
      },
    })
    vi.spyOn(HTMLMediaElement.prototype, 'play').mockImplementation(function (this: HTMLMediaElement) {
      ;(this as HTMLMediaElement & { _paused?: boolean })._paused = false
      this.dispatchEvent(new Event('playing'))
      return Promise.resolve()
    })
    vi.spyOn(HTMLMediaElement.prototype, 'pause').mockImplementation(function (this: HTMLMediaElement) {
      ;(this as HTMLMediaElement & { _paused?: boolean })._paused = true
      this.dispatchEvent(new Event('pause'))
    })
    vi.spyOn(HTMLMediaElement.prototype, 'load').mockImplementation(() => undefined)
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('I5: uses native playback for HLS when the browser can play it', () => {
    const onAck = vi.fn()
    render(
      <AvProjectedLivestream
        command={liveCommand(1)}
        onAck={onAck}
        nativeHlsSupported={() => true}
        importHls={async () => {
          throw new Error('should not load hls.js')
        }}
      />,
    )
    const video = screen.getByTestId('av-projected-livestream-video') as HTMLVideoElement
    expect(video.getAttribute('src')).toBe('https://example.com/live.m3u8')
    expect(HTMLMediaElement.prototype.play).toHaveBeenCalled()
  })

  it('I5: loads hls.js only when native HLS is unavailable', async () => {
    const loadSource = vi.fn()
    const attachMedia = vi.fn()
    const destroy = vi.fn()
    const on = vi.fn()
    const nativeHlsSupported = () => false
    const importHls = vi.fn(async (): Promise<HlsModule> => ({
      default: Object.assign(
        class {
          loadSource = loadSource
          attachMedia = attachMedia
          destroy = destroy
          on = on
        },
        { isSupported: () => true, Events: { ERROR: 'hlsError', MANIFEST_PARSED: 'manifest' } },
      ),
    }))
    render(
      <AvProjectedLivestream
        command={liveCommand(1)}
        onAck={vi.fn()}
        nativeHlsSupported={nativeHlsSupported}
        importHls={importHls}
      />,
    )
    await vi.waitFor(() => expect(importHls).toHaveBeenCalled())
    await vi.waitFor(() => expect(loadSource).toHaveBeenCalledWith('https://example.com/live.m3u8'))
    expect(attachMedia).toHaveBeenCalled()
  })

  it('I5: reports unsupported when hls.js cannot run', async () => {
    const onAck = vi.fn()
    const nativeHlsSupported = () => false
    const importHls = async (): Promise<HlsModule> => ({
      default: Object.assign(
        class {
          loadSource = vi.fn()
          attachMedia = vi.fn()
          destroy = vi.fn()
          on = vi.fn()
        },
        { isSupported: () => false, Events: { ERROR: 'hlsError', MANIFEST_PARSED: 'manifest' } },
      ),
    })
    render(
      <AvProjectedLivestream
        command={liveCommand(1)}
        onAck={onAck}
        nativeHlsSupported={nativeHlsSupported}
        importHls={importHls}
      />,
    )
    await vi.waitFor(() =>
      expect(onAck).toHaveBeenCalledWith(
        false,
        undefined,
        expect.objectContaining({ code: 'unsupported_source' }),
      ),
    )
  })
})
