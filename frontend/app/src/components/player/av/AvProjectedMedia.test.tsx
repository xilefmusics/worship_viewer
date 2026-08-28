import { act, render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { AvProjectedMedia } from '@/components/player/av/AvProjectedMedia'
import { DEFAULT_AV_PREFERENCES } from '@/lib/player/av-preferences'
import { buildAvPlaybackIntent } from '@/lib/player/av-projection-playback'
import { buildAvProjectionCommand } from '@/lib/player/av-projection-protocol'

const layers = {
  contentLayer: DEFAULT_AV_PREFERENCES.contentLayer,
  backgroundLayer: DEFAULT_AV_PREFERENCES.backgroundLayer,
  transition: DEFAULT_AV_PREFERENCES.transition,
}

function videoCommand(
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
    content: { type: 'video', mediaId: 'media-2', assetId: 'v1' },
    playback,
  })
}

describe('AvProjectedMedia', () => {
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

  it('I4: uses the authenticated range URL and plays video contained over the stage', () => {
    const onAck = vi.fn()
    render(<AvProjectedMedia command={videoCommand(1)} onAck={onAck} />)
    const video = screen.getByTestId('av-projected-video') as HTMLVideoElement
    expect(video.getAttribute('src')).toContain('/api/v1/media/media-2/assets/v1/data')
    expect(HTMLMediaElement.prototype.play).toHaveBeenCalled()
    expect(screen.getByTestId('av-projected-media')).not.toHaveClass('av-projected-media--hidden')
  })

  it('I4: hides audio visually while still using the range URL', () => {
    const onAck = vi.fn()
    render(
      <AvProjectedMedia
        command={buildAvProjectionCommand({
          sessionId: 'shared',
          commandId: 1,
          ...layers,
          screenState: 'live',
          itemTitle: 'Track',
          nextPreview: null,
          content: { type: 'audio', mediaId: 'media-3', assetId: 'a1' },
          playback: buildAvPlaybackIntent({ action: 'play' }),
        })}
        onAck={onAck}
      />,
    )
    const audio = screen.getByTestId('av-projected-audio')
    expect(audio.getAttribute('src')).toContain('/api/v1/media/media-3/assets/a1/data')
    expect(screen.getByTestId('av-projected-media')).toHaveClass('av-projected-media--hidden')
  })

  it('I4: pauses and hides on blank without releasing the element', () => {
    const onAck = vi.fn()
    const { rerender } = render(<AvProjectedMedia command={videoCommand(1)} onAck={onAck} />)
    const video = screen.getByTestId('av-projected-video')
    const src = video.getAttribute('src')
    rerender(
      <AvProjectedMedia
        command={videoCommand(2, buildAvPlaybackIntent({ action: 'play' }), 'blank')}
        onAck={onAck}
      />,
    )
    expect(HTMLMediaElement.prototype.pause).toHaveBeenCalled()
    expect(screen.getByTestId('av-projected-media')).toHaveClass('av-projected-media--hidden')
    expect(screen.getByTestId('av-projected-video').getAttribute('src')).toBe(src)
  })

  it('I4: loops instead of ending when loop is on', () => {
    const onAck = vi.fn()
    render(
      <AvProjectedMedia
        command={videoCommand(1, buildAvPlaybackIntent({ action: 'play', loop: true }))}
        onAck={onAck}
      />,
    )
    const video = screen.getByTestId('av-projected-video') as HTMLVideoElement
    act(() => {
      video.dispatchEvent(new Event('ended'))
    })
    expect(HTMLMediaElement.prototype.play).toHaveBeenCalled()
    const endedAcks = onAck.mock.calls.filter((call) => call[1]?.status === 'ended')
    expect(endedAcks).toHaveLength(0)
  })

  it('I4: clears the visual layer on ended and does not keep a last frame', () => {
    const onAck = vi.fn()
    render(<AvProjectedMedia command={videoCommand(1)} onAck={onAck} />)
    const video = screen.getByTestId('av-projected-video') as HTMLVideoElement
    act(() => {
      video.dispatchEvent(new Event('ended'))
    })
    expect(video.getAttribute('src')).toBeNull()
    expect(screen.getByTestId('av-projected-media')).toHaveClass('av-projected-media--hidden')
    expect(onAck).toHaveBeenCalledWith(true, expect.objectContaining({ status: 'ended' }), undefined)
  })

  it('I4: returning to Live with pause does not auto-resume', () => {
    const onAck = vi.fn()
    const { rerender } = render(<AvProjectedMedia command={videoCommand(1)} onAck={onAck} />)
    expect(HTMLMediaElement.prototype.play).toHaveBeenCalled()
    vi.mocked(HTMLMediaElement.prototype.play).mockClear()
    rerender(
      <AvProjectedMedia
        command={videoCommand(2, buildAvPlaybackIntent({ action: 'pause' }), 'blank')}
        onAck={onAck}
      />,
    )
    rerender(
      <AvProjectedMedia
        command={videoCommand(3, buildAvPlaybackIntent({ action: 'pause' }), 'live')}
        onAck={onAck}
      />,
    )
    expect(HTMLMediaElement.prototype.play).not.toHaveBeenCalled()
    expect(HTMLMediaElement.prototype.pause).toHaveBeenCalled()
    expect(screen.getByTestId('av-projected-media')).not.toHaveClass('av-projected-media--hidden')
  })

  it('I4: reports autoplay failure without keeping a visible last frame', async () => {
    vi.spyOn(HTMLMediaElement.prototype, 'play').mockImplementation(() =>
      Promise.reject(new Error('NotAllowedError')),
    )
    const onAck = vi.fn()
    render(<AvProjectedMedia command={videoCommand(1)} onAck={onAck} />)
    await act(async () => {
      await Promise.resolve()
    })
    expect(onAck).toHaveBeenCalledWith(
      false,
      expect.objectContaining({ status: 'error' }),
      expect.objectContaining({ code: 'autoplay_blocked' }),
    )
  })
})
