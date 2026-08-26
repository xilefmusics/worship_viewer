import { describe, expect, it } from 'vitest'

import { DEFAULT_AV_PREFERENCES } from '@/lib/player/av-preferences'
import {
  AV_PLAYBACK_ACK_MIN_INTERVAL_MS,
  AV_PLAYBACK_ERROR_AUTOPLAY,
  buildAvPlaybackIntent,
  clampAvPositionMs,
  clampAvVolume,
  formatAvClock,
  mediaActionsForCommand,
  playbackAckChangedMaterially,
  playbackAckFromElement,
  playbackError,
  shouldEmitPlaybackAck,
} from '@/lib/player/av-projection-playback'
import { buildAvProjectionCommand } from '@/lib/player/av-projection-protocol'
import {
  INITIAL_CONTROLLER_PROJECTION_STATE,
  aggregateAvPlayback,
  applyOutputCommand,
  reduceControllerProjection,
  type AvOutputProjectionState,
} from '@/lib/player/av-projection-reducer'

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
    content: { type: 'video', mediaId: 'm1', assetId: 'v1' },
    playback,
  })
}

function audioCommand(commandId: number, playback = buildAvPlaybackIntent({ action: 'play' })) {
  return buildAvProjectionCommand({
    sessionId: 'shared',
    commandId,
    ...layers,
    screenState: 'live',
    itemTitle: 'Track',
    nextPreview: null,
    content: { type: 'audio', mediaId: 'm1', assetId: 'a1' },
    playback,
  })
}

describe('playback intent validation', () => {
  it('clamps volume and seek and rejects NaN', () => {
    expect(clampAvVolume(1.5)).toBe(1)
    expect(clampAvVolume(-0.2)).toBe(0)
    expect(clampAvVolume(Number.NaN)).toBe(1)
    expect(clampAvPositionMs(-12, 0, 5000)).toBe(0)
    expect(clampAvPositionMs(9000, 0, 5000)).toBe(5000)
    expect(clampAvPositionMs(Number.NaN, 0, 5000)).toBeUndefined()
  })

  it('builds every transport action with required fields', () => {
    const play = buildAvPlaybackIntent({ action: 'play', issuedAtMs: 10 })
    expect(play).toMatchObject({
      action: 'play',
      volume: 1,
      muted: false,
      loop: false,
      issuedAtMs: 10,
    })
    expect(buildAvPlaybackIntent({ action: 'pause' }).action).toBe('pause')
    expect(buildAvPlaybackIntent({ action: 'resume' }).action).toBe('resume')
    expect(buildAvPlaybackIntent({ action: 'seek', positionMs: 1500 }).positionMs).toBe(1500)
    expect(buildAvPlaybackIntent({ action: 'restart' }).positionMs).toBe(0)
    expect(
      buildAvPlaybackIntent({ action: 'configure', volume: 0.4, muted: true, loop: true }),
    ).toMatchObject({ action: 'configure', volume: 0.4, muted: true, loop: true })
    expect(formatAvClock(125000)).toBe('2:05')
  })
})

describe('output media actions', () => {
  it('plays on play, pauses on pause, and seeks without forcing pause', () => {
    expect(mediaActionsForCommand(videoCommand(1))?.play).toBe(true)
    expect(
      mediaActionsForCommand(videoCommand(2, buildAvPlaybackIntent({ action: 'pause' })))?.pause,
    ).toBe(true)
    const seek = mediaActionsForCommand(
      videoCommand(3, buildAvPlaybackIntent({ action: 'seek', positionMs: 1200 })),
      4000,
      4000,
    )
    expect(seek?.play).toBe(false)
    expect(seek?.pause).toBe(false)
    expect(seek?.seekMs).toBe(1200)
  })

  it('pauses and hides on blank or blackout even when the intent is play', () => {
    const blank = mediaActionsForCommand(videoCommand(1, buildAvPlaybackIntent({ action: 'play' }), 'blank'))
    expect(blank?.play).toBe(false)
    expect(blank?.pause).toBe(true)
    expect(blank?.hide).toBe(true)
    expect(blank?.release).toBe(false)
  })

  it('releases media on clear or lyric replacement', () => {
    const clear = mediaActionsForCommand(
      buildAvProjectionCommand({
        sessionId: 'shared',
        commandId: 2,
        intent: 'clear',
        ...layers,
        screenState: 'live',
        itemTitle: 'Clip',
        nextPreview: null,
        content: { type: 'video', mediaId: 'm1', assetId: 'v1' },
      }),
    )
    expect(clear?.release).toBe(true)
  })

  it('restarts from the start and plays when live', () => {
    const restart = mediaActionsForCommand(
      videoCommand(4, buildAvPlaybackIntent({ action: 'restart' })),
    )
    expect(restart?.seekMs).toBe(0)
    expect(restart?.play).toBe(true)
  })
})

describe('playback acks', () => {
  it('builds a clamped element snapshot', () => {
    const ack = playbackAckFromElement({
      status: 'playing',
      currentTimeMs: 2500,
      durationMs: 4000,
      volume: 0.8,
      muted: false,
      loop: true,
    })
    expect(ack).toEqual({
      status: 'playing',
      currentTimeMs: 2500,
      durationMs: 4000,
      seekableStartMs: 0,
      seekableEndMs: 4000,
      volume: 0.8,
      muted: false,
      loop: true,
    })
  })

  it('throttles time-only acks and always emits status changes', () => {
    const playing = playbackAckFromElement({
      status: 'playing',
      currentTimeMs: 0,
      durationMs: 1000,
      volume: 1,
      muted: false,
      loop: false,
    })
    const later = { ...playing, currentTimeMs: 200 }
    expect(shouldEmitPlaybackAck({ at: 0, ack: playing }, later, 10)).toBe(false)
    expect(
      shouldEmitPlaybackAck({ at: 0, ack: playing }, later, AV_PLAYBACK_ACK_MIN_INTERVAL_MS),
    ).toBe(true)
    expect(
      shouldEmitPlaybackAck(
        { at: 0, ack: playing },
        { ...playing, status: 'paused' },
        10,
      ),
    ).toBe(true)
    expect(playbackAckChangedMaterially(playing, later)).toBe(false)
  })

  it('names autoplay failures as a safe error code', () => {
    expect(playbackError(AV_PLAYBACK_ERROR_AUTOPLAY, 'blocked').code).toBe('autoplay_blocked')
  })
})

describe('controller playback aggregation', () => {
  function withHello(commandId: number) {
    let state = reduceControllerProjection(
      INITIAL_CONTROLLER_PROJECTION_STATE,
      { type: 'issue', command: videoCommand(commandId) },
      0,
    )
    state = reduceControllerProjection(
      state,
      { type: 'message', message: { type: 'hello', sessionId: 'shared', outputId: 'a', ready: true } },
      1,
    )
    state = reduceControllerProjection(
      state,
      { type: 'message', message: { type: 'hello', sessionId: 'shared', outputId: 'b', ready: true } },
      1,
    )
    return state
  }

  it('ignores stale playback acks after command replacement', () => {
    let state = withHello(1)
    state = reduceControllerProjection(
      state,
      {
        type: 'message',
        message: {
          type: 'ack',
          sessionId: 'shared',
          commandId: 1,
          outputId: 'a',
          applied: true,
          current: { screenState: 'live', content: { type: 'video', mediaId: 'm1', assetId: 'v1' } },
          playback: playbackAckFromElement({
            status: 'playing',
            currentTimeMs: 100,
            durationMs: 1000,
            volume: 1,
            muted: false,
            loop: false,
          }),
        },
      },
      2,
    )
    state = reduceControllerProjection(
      state,
      { type: 'issue', command: videoCommand(2, buildAvPlaybackIntent({ action: 'pause' })) },
      3,
    )
    state = reduceControllerProjection(
      state,
      {
        type: 'message',
        message: {
          type: 'ack',
          sessionId: 'shared',
          commandId: 1,
          outputId: 'a',
          applied: false,
          current: { screenState: 'live', content: { type: 'video', mediaId: 'm1', assetId: 'v1' } },
          error: { code: 'stale', detail: 'old' },
          playback: playbackAckFromElement({
            status: 'error',
            currentTimeMs: 100,
            durationMs: 1000,
            volume: 1,
            muted: false,
            loop: false,
          }),
        },
      },
      4,
    )
    expect(state.outputs.a?.error).toBeUndefined()
    expect(state.outputs.a?.playback?.status).toBe('playing')
  })

  it('clears playback when replacing with a different item so retry cannot attach stale acks', () => {
    let state = withHello(1)
    state = reduceControllerProjection(
      state,
      {
        type: 'message',
        message: {
          type: 'ack',
          sessionId: 'shared',
          commandId: 1,
          outputId: 'a',
          applied: false,
          current: { screenState: 'live', content: { type: 'video', mediaId: 'm1', assetId: 'v1' } },
          error: { code: AV_PLAYBACK_ERROR_AUTOPLAY, detail: 'blocked' },
        },
      },
      2,
    )
    state = reduceControllerProjection(
      state,
      { type: 'issue', command: audioCommand(2) },
      3,
    )
    expect(state.outputs.a?.playback).toBeUndefined()
    expect(state.outputs.a?.error).toBeUndefined()
  })

  it('aggregates divergent outputs without averaging clocks', () => {
    let state = withHello(1)
    const playing = playbackAckFromElement({
      status: 'playing',
      currentTimeMs: 800,
      durationMs: 4000,
      volume: 0.5,
      muted: false,
      loop: false,
    })
    const paused = playbackAckFromElement({
      status: 'paused',
      currentTimeMs: 1200,
      durationMs: 4000,
      volume: 0.5,
      muted: false,
      loop: false,
    })
    state = reduceControllerProjection(
      state,
      {
        type: 'message',
        message: {
          type: 'ack',
          sessionId: 'shared',
          commandId: 1,
          outputId: 'a',
          applied: true,
          current: { screenState: 'live', content: { type: 'video', mediaId: 'm1', assetId: 'v1' } },
          playback: playing,
        },
      },
      5,
    )
    state = reduceControllerProjection(
      state,
      {
        type: 'message',
        message: {
          type: 'ack',
          sessionId: 'shared',
          commandId: 1,
          outputId: 'b',
          applied: true,
          current: { screenState: 'live', content: { type: 'video', mediaId: 'm1', assetId: 'v1' } },
          playback: paused,
        },
      },
      5,
    )
    const aggregate = aggregateAvPlayback(state)
    expect(aggregate.status).toBe('playing')
    expect(aggregate.currentTimeMs).toBe(800)
    expect(aggregate.playingCount).toBe(1)
    expect(aggregate.pausedCount).toBe(1)
    expect(aggregate.outputs).toHaveLength(2)
  })

  it('marks a failed output without stopping a successful sibling', () => {
    let state = withHello(1)
    state = reduceControllerProjection(
      state,
      {
        type: 'message',
        message: {
          type: 'ack',
          sessionId: 'shared',
          commandId: 1,
          outputId: 'a',
          applied: false,
          current: { screenState: 'live', content: { type: 'video', mediaId: 'm1', assetId: 'v1' } },
          error: { code: AV_PLAYBACK_ERROR_AUTOPLAY, detail: 'blocked' },
          playback: playbackAckFromElement({
            status: 'error',
            currentTimeMs: 0,
            durationMs: null,
            volume: 1,
            muted: false,
            loop: false,
          }),
        },
      },
      2,
    )
    state = reduceControllerProjection(
      state,
      {
        type: 'message',
        message: {
          type: 'ack',
          sessionId: 'shared',
          commandId: 1,
          outputId: 'b',
          applied: true,
          current: { screenState: 'live', content: { type: 'video', mediaId: 'm1', assetId: 'v1' } },
          playback: playbackAckFromElement({
            status: 'playing',
            currentTimeMs: 40,
            durationMs: 1000,
            volume: 1,
            muted: false,
            loop: false,
          }),
        },
      },
      2,
    )
    expect(state.outputs.a?.status).toBe('failed')
    expect(state.outputs.b?.status).toBe('ready')
    expect(aggregateAvPlayback(state).status).toBe('playing')
  })
})

describe('output command apply for timed media', () => {
  const base: AvOutputProjectionState = { outputId: 'out-1', appliedCommandId: null, command: null }

  it('applies same-content playback replacement without cleanup', () => {
    const play = applyOutputCommand(base, videoCommand(1))
    const pause = applyOutputCommand(
      play.state,
      videoCommand(2, buildAvPlaybackIntent({ action: 'pause' })),
    )
    expect(pause.applied).toBe(true)
    expect(pause.previousContent).toBeNull()
    expect(pause.state.command?.playback?.action).toBe('pause')
  })

  it('requests cleanup when replacing video with lyrics', () => {
    const play = applyOutputCommand(base, videoCommand(1))
    const lyrics = applyOutputCommand(
      play.state,
      buildAvProjectionCommand({
        sessionId: 'shared',
        commandId: 2,
        ...layers,
        screenState: 'live',
        itemTitle: 'Song',
        nextPreview: null,
        content: { type: 'lyrics', contentText: 'Hello' },
      }),
    )
    expect(lyrics.previousContent).toEqual({ type: 'video', mediaId: 'm1', assetId: 'v1' })
  })
})
