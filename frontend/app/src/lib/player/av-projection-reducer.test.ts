import { describe, expect, it } from 'vitest'

import { DEFAULT_AV_PREFERENCES } from '@/lib/player/av-preferences'
import { buildAvProjectionCommand } from '@/lib/player/av-projection-protocol'
import {
  AV_OUTPUT_MISSING_AFTER_MS,
  INITIAL_CONTROLLER_PROJECTION_STATE,
  applyOutputCommand,
  hasReadyAvOutput,
  nextAvProjectionCommandId,
  outputAckForApply,
  reduceControllerProjection,
  summarizeAvOutputs,
  type AvOutputProjectionState,
} from '@/lib/player/av-projection-reducer'

const layers = {
  contentLayer: DEFAULT_AV_PREFERENCES.contentLayer,
  backgroundLayer: DEFAULT_AV_PREFERENCES.backgroundLayer,
  transition: DEFAULT_AV_PREFERENCES.transition,
}

function lyricsCommand(commandId: number, text = 'Hello') {
  return buildAvProjectionCommand({
    sessionId: 'shared',
    commandId,
    ...layers,
    screenState: 'live',
    itemTitle: 'Song',
    nextPreview: null,
    content: { type: 'lyrics', contentText: text },
  })
}

function deckCommand(commandId: number, assetId = 'a1') {
  return buildAvProjectionCommand({
    sessionId: 'shared',
    commandId,
    ...layers,
    screenState: 'live',
    itemTitle: 'Deck',
    nextPreview: null,
    content: { type: 'deck_page', mediaId: 'm1', assetId },
  })
}

describe('controller projection reducer', () => {
  it('tracks command ids and ignores stale or duplicate acks', () => {
    let state = reduceControllerProjection(
      INITIAL_CONTROLLER_PROJECTION_STATE,
      { type: 'issue', command: lyricsCommand(1) },
      0,
    )
    expect(state.latestCommandId).toBe(1)
    expect(nextAvProjectionCommandId(state)).toBe(2)

    state = reduceControllerProjection(
      state,
      {
        type: 'message',
        message: {
          type: 'hello',
          sessionId: 'shared',
          outputId: 'out-1',
          ready: true,
        },
      },
      10,
    )
    state = reduceControllerProjection(
      state,
      {
        type: 'message',
        message: {
          type: 'ack',
          sessionId: 'shared',
          commandId: 1,
          outputId: 'out-1',
          applied: true,
          current: { screenState: 'live', content: { type: 'lyrics', contentText: 'Hello' } },
        },
      },
      20,
    )
    expect(state.outputs['out-1']?.lastAckCommandId).toBe(1)

    state = reduceControllerProjection(
      state,
      { type: 'issue', command: lyricsCommand(2, 'World') },
      30,
    )
    state = reduceControllerProjection(
      state,
      {
        type: 'message',
        message: {
          type: 'ack',
          sessionId: 'shared',
          commandId: 1,
          outputId: 'out-1',
          applied: false,
          current: { screenState: 'live', content: { type: 'lyrics', contentText: 'Hello' } },
          error: { code: 'stale', detail: 'old' },
        },
      },
      40,
    )
    expect(state.outputs['out-1']?.error).toBeUndefined()
    expect(state.outputs['out-1']?.status).toBe('ready')
  })

  it('tracks multiple outputs independently and does not fail siblings', () => {
    let state = reduceControllerProjection(
      INITIAL_CONTROLLER_PROJECTION_STATE,
      { type: 'issue', command: deckCommand(1) },
      0,
    )
    state = reduceControllerProjection(
      state,
      { type: 'message', message: { type: 'hello', sessionId: 'shared', outputId: 'a', ready: true } },
      5,
    )
    state = reduceControllerProjection(
      state,
      { type: 'message', message: { type: 'hello', sessionId: 'shared', outputId: 'b', ready: true } },
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
          outputId: 'a',
          applied: false,
          current: { screenState: 'live', content: { type: 'deck_page', mediaId: 'm1', assetId: 'a1' } },
          error: { code: 'asset_failed', detail: 'fetch failed' },
        },
      },
      10,
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
          current: { screenState: 'live', content: { type: 'deck_page', mediaId: 'm1', assetId: 'a1' } },
        },
      },
      10,
    )

    expect(state.outputs.a?.status).toBe('failed')
    expect(state.outputs.b?.status).toBe('ready')
    expect(summarizeAvOutputs(state)).toEqual({ ready: 1, missing: 0, failed: 1, total: 2 })
    expect(hasReadyAvOutput(state)).toBe(true)
  })

  it('marks outputs missing after the heartbeat timeout without dropping the latest command', () => {
    let state = reduceControllerProjection(
      INITIAL_CONTROLLER_PROJECTION_STATE,
      { type: 'issue', command: lyricsCommand(1) },
      0,
    )
    state = reduceControllerProjection(
      state,
      { type: 'message', message: { type: 'hello', sessionId: 'shared', outputId: 'out-1', ready: true } },
      0,
    )
    expect(hasReadyAvOutput(state)).toBe(true)

    state = reduceControllerProjection(state, { type: 'tick' }, AV_OUTPUT_MISSING_AFTER_MS + 1)
    expect(state.outputs['out-1']?.status).toBe('missing')
    expect(state.latestCommand?.content).toEqual({ type: 'lyrics', contentText: 'Hello' })
    expect(hasReadyAvOutput(state)).toBe(false)
  })

  it('removes an output on goodbye', () => {
    let state = reduceControllerProjection(
      INITIAL_CONTROLLER_PROJECTION_STATE,
      { type: 'message', message: { type: 'hello', sessionId: 'shared', outputId: 'out-1', ready: true } },
      0,
    )
    state = reduceControllerProjection(
      state,
      { type: 'message', message: { type: 'goodbye', sessionId: 'shared', outputId: 'out-1', ready: false } },
      1,
    )
    expect(state.outputs['out-1']).toBeUndefined()
    expect(summarizeAvOutputs(state).total).toBe(0)
  })
})

describe('output projection reducer', () => {
  const base: AvOutputProjectionState = { outputId: 'out-1', appliedCommandId: null, command: null }

  it('applies newer commands and reports previous content for cleanup', () => {
    const first = applyOutputCommand(base, deckCommand(1, 'a1'))
    expect(first.applied).toBe(true)
    expect(first.previousContent).toBeNull()

    const second = applyOutputCommand(first.state, lyricsCommand(2, 'Hello'))
    expect(second.applied).toBe(true)
    expect(second.previousContent).toEqual({ type: 'deck_page', mediaId: 'm1', assetId: 'a1' })
  })

  it('ignores stale and duplicate commands', () => {
    const applied = applyOutputCommand(base, lyricsCommand(5))
    expect(applyOutputCommand(applied.state, lyricsCommand(4)).applied).toBe(false)
    expect(applyOutputCommand(applied.state, lyricsCommand(5)).applied).toBe(false)
  })

  it('clear replaces content with empty and requests cleanup', () => {
    const applied = applyOutputCommand(base, deckCommand(1))
    const cleared = applyOutputCommand(
      applied.state,
      buildAvProjectionCommand({
        sessionId: 'shared',
        commandId: 2,
        intent: 'clear',
        ...layers,
        screenState: 'live',
        itemTitle: 'Deck',
        nextPreview: null,
        content: { type: 'deck_page', mediaId: 'm1', assetId: 'a1' },
      }),
    )
    expect(cleared.state.command?.content).toEqual({ type: 'empty' })
    expect(cleared.previousContent).toEqual({ type: 'deck_page', mediaId: 'm1', assetId: 'a1' })
  })

  it('does not request cleanup when only screen state changes', () => {
    const live = applyOutputCommand(base, deckCommand(1))
    const blank = applyOutputCommand(
      live.state,
      { ...deckCommand(2), screenState: 'blank' },
    )
    expect(blank.applied).toBe(true)
    expect(blank.previousContent).toBeNull()
  })

  it('builds an ack from the applied command', () => {
    const applied = applyOutputCommand(base, lyricsCommand(3))
    expect(outputAckForApply(applied.state, true)).toMatchObject({
      type: 'ack',
      commandId: 3,
      outputId: 'out-1',
      applied: true,
    })
  })
})
