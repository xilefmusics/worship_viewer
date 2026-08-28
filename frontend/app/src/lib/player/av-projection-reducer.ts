import {
  sameAvProjectionContent,
  type AvPlaybackStatus,
  type AvProjectionAck,
  type AvProjectionAckError,
  type AvProjectionCommand,
  type AvProjectionCommandId,
  type AvProjectionContent,
  type AvProjectionMessage,
  type AvProjectionPlaybackAck,
  type AvProjectionPresence,
} from '@/lib/player/av-projection-protocol'
import {
  DEFAULT_AV_PLAYBACK_LOOP,
  DEFAULT_AV_PLAYBACK_MUTED,
  DEFAULT_AV_PLAYBACK_VOLUME,
} from '@/lib/player/av-projection-playback'

export const AV_OUTPUT_HEARTBEAT_MS = 1000
export const AV_OUTPUT_MISSING_AFTER_MS = 3500

export type AvOutputStatus = 'ready' | 'missing' | 'failed'

export type AvTrackedOutput = {
  outputId: string
  ready: boolean
  status: AvOutputStatus
  lastSeenAt: number
  lastAckCommandId: AvProjectionCommandId | null
  error?: AvProjectionAckError
  playback?: AvProjectionPlaybackAck
}

export type AvControllerProjectionState = {
  latestCommandId: AvProjectionCommandId
  latestCommand: AvProjectionCommand | null
  outputs: Record<string, AvTrackedOutput>
}

export const INITIAL_CONTROLLER_PROJECTION_STATE: AvControllerProjectionState = {
  latestCommandId: 0,
  latestCommand: null,
  outputs: {},
}

export type AvControllerProjectionEvent =
  | { type: 'issue'; command: AvProjectionCommand }
  | { type: 'message'; message: AvProjectionMessage }
  | { type: 'tick' }

export type AvOutputSummary = {
  ready: number
  missing: number
  failed: number
  total: number
}

export function nextAvProjectionCommandId(
  state: AvControllerProjectionState,
): AvProjectionCommandId {
  return state.latestCommandId + 1
}

export function summarizeAvOutputs(state: AvControllerProjectionState): AvOutputSummary {
  let ready = 0
  let missing = 0
  let failed = 0
  for (const output of Object.values(state.outputs)) {
    if (output.status === 'ready') ready += 1
    else if (output.status === 'failed') failed += 1
    else missing += 1
  }
  return { ready, missing, failed, total: ready + missing + failed }
}

export function hasReadyAvOutput(state: AvControllerProjectionState): boolean {
  return summarizeAvOutputs(state).ready > 0
}

function refreshOutputStatus(output: AvTrackedOutput, now: number): AvTrackedOutput {
  if (now - output.lastSeenAt > AV_OUTPUT_MISSING_AFTER_MS) {
    return { ...output, ready: false, status: 'missing' }
  }
  if (output.error) return { ...output, status: 'failed' }
  return { ...output, ready: true, status: 'ready' }
}

function upsertPresence(
  state: AvControllerProjectionState,
  message: AvProjectionPresence,
  now: number,
): AvControllerProjectionState {
  if (message.type === 'goodbye') {
    const outputs = { ...state.outputs }
    delete outputs[message.outputId]
    return { ...state, outputs }
  }
  const previous = state.outputs[message.outputId]
  const next: AvTrackedOutput = refreshOutputStatus(
    {
      outputId: message.outputId,
      ready: message.ready,
      status: message.ready ? 'ready' : 'missing',
      lastSeenAt: now,
      lastAckCommandId: previous?.lastAckCommandId ?? null,
      error: previous?.error,
      playback: previous?.playback,
    },
    now,
  )
  return {
    ...state,
    outputs: { ...state.outputs, [message.outputId]: next },
  }
}

function applyAck(
  state: AvControllerProjectionState,
  ack: AvProjectionAck,
  now: number,
): AvControllerProjectionState {
  if (ack.commandId !== state.latestCommandId) return state
  const previous = state.outputs[ack.outputId]
  const base: AvTrackedOutput = previous ?? {
    outputId: ack.outputId,
    ready: true,
    status: 'ready',
    lastSeenAt: now,
    lastAckCommandId: null,
  }
  const error = ack.applied ? undefined : ack.error
  const next = refreshOutputStatus(
    {
      ...base,
      lastSeenAt: now,
      lastAckCommandId: ack.commandId,
      error,
      playback: ack.playback ?? base.playback,
    },
    now,
  )
  return {
    ...state,
    outputs: { ...state.outputs, [ack.outputId]: next },
  }
}

export function reduceControllerProjection(
  state: AvControllerProjectionState,
  event: AvControllerProjectionEvent,
  now: number,
): AvControllerProjectionState {
  if (event.type === 'issue') {
    const contentChanged =
      state.latestCommand != null &&
      !sameAvProjectionContent(state.latestCommand.content, event.command.content)
    const outputs: Record<string, AvTrackedOutput> = {}
    for (const [id, output] of Object.entries(state.outputs)) {
      outputs[id] = refreshOutputStatus(
        {
          ...output,
          error: undefined,
          lastAckCommandId: output.lastAckCommandId,
          playback: contentChanged ? undefined : output.playback,
        },
        now,
      )
    }
    return {
      latestCommandId: event.command.commandId,
      latestCommand: event.command,
      outputs,
    }
  }

  if (event.type === 'tick') {
    const outputs: Record<string, AvTrackedOutput> = {}
    for (const [id, output] of Object.entries(state.outputs)) {
      outputs[id] = refreshOutputStatus(output, now)
    }
    return { ...state, outputs }
  }

  const { message } = event
  if (message.type === 'command') return state
  if (message.type === 'ack') return applyAck(state, message, now)
  return upsertPresence(state, message, now)
}

export type AvOutputProjectionState = {
  outputId: string
  appliedCommandId: AvProjectionCommandId | null
  command: AvProjectionCommand | null
}

export const INITIAL_OUTPUT_PROJECTION_STATE: Omit<AvOutputProjectionState, 'outputId'> = {
  appliedCommandId: null,
  command: null,
}

export type AvOutputApplyResult = {
  state: AvOutputProjectionState
  applied: boolean
  previousContent: AvProjectionContent | null
}

export function applyOutputCommand(
  state: AvOutputProjectionState,
  command: AvProjectionCommand,
): AvOutputApplyResult {
  if (state.appliedCommandId != null && command.commandId < state.appliedCommandId) {
    return { state, applied: false, previousContent: null }
  }
  if (state.appliedCommandId === command.commandId) {
    return { state, applied: false, previousContent: null }
  }
  const previous = state.command?.content ?? null
  const nextContent = command.intent === 'clear' ? { type: 'empty' as const } : command.content
  const shouldCleanup =
    previous != null && !sameAvProjectionContent(previous, nextContent)
  return {
    state: {
      outputId: state.outputId,
      appliedCommandId: command.commandId,
      command: { ...command, content: nextContent },
    },
    applied: true,
    previousContent: shouldCleanup ? previous : null,
  }
}

export function outputAckForApply(
  state: AvOutputProjectionState,
  applied: boolean,
  error?: AvProjectionAckError,
  playback?: AvProjectionPlaybackAck,
): AvProjectionAck | null {
  if (!state.command || state.appliedCommandId == null) return null
  return {
    type: 'ack',
    sessionId: state.command.sessionId,
    commandId: state.appliedCommandId,
    outputId: state.outputId,
    applied,
    current: {
      screenState: state.command.screenState,
      content: state.command.content,
    },
    ...(error ? { error } : {}),
    ...(playback ? { playback } : {}),
  }
}

export type AvAggregatedPlaybackStatus = AvPlaybackStatus | 'mixed' | 'idle'

export type AvAggregatedPlayback = {
  status: AvAggregatedPlaybackStatus
  currentTimeMs: number
  durationMs: number | null
  seekableStartMs: number
  seekableEndMs: number
  volume: number
  muted: boolean
  loop: boolean
  playingCount: number
  pausedCount: number
  endedCount: number
  errorCount: number
  loadingCount: number
  outputs: Array<{
    outputId: string
    outputStatus: AvTrackedOutput['status']
    playback?: AvProjectionPlaybackAck
    error?: AvTrackedOutput['error']
  }>
}

function aggregatedPlaybackStatus(counts: {
  clocksLength: number
  playingCount: number
  pausedCount: number
  endedCount: number
  errorCount: number
  loadingCount: number
}): AvAggregatedPlaybackStatus {
  if (counts.playingCount > 0) return 'playing'
  if (counts.clocksLength === 0) return 'idle'
  if (counts.errorCount > 0 && counts.pausedCount === 0 && counts.endedCount === 0 && counts.loadingCount === 0) {
    return 'error'
  }
  if (counts.endedCount > 0 && counts.playingCount === 0 && counts.pausedCount === 0 && counts.loadingCount === 0) {
    return counts.endedCount === counts.clocksLength ? 'ended' : 'mixed'
  }
  if (counts.pausedCount > 0 && counts.endedCount === 0 && counts.errorCount === 0 && counts.loadingCount === 0) {
    return 'paused'
  }
  if (counts.loadingCount > 0 && counts.playingCount === 0 && counts.pausedCount === 0 && counts.endedCount === 0) {
    return 'loading'
  }
  if (counts.pausedCount > 0) return 'paused'
  return 'mixed'
}

const IDLE_PLAYBACK: AvAggregatedPlayback = {
  status: 'idle',
  currentTimeMs: 0,
  durationMs: null,
  seekableStartMs: 0,
  seekableEndMs: 0,
  volume: DEFAULT_AV_PLAYBACK_VOLUME,
  muted: DEFAULT_AV_PLAYBACK_MUTED,
  loop: DEFAULT_AV_PLAYBACK_LOOP,
  playingCount: 0,
  pausedCount: 0,
  endedCount: 0,
  errorCount: 0,
  loadingCount: 0,
  outputs: [],
}

export function aggregateAvPlayback(
  state: AvControllerProjectionState,
): AvAggregatedPlayback {
  const outputs = Object.values(state.outputs).map((output) => ({
    outputId: output.outputId,
    outputStatus: output.status,
    playback: output.playback,
    error: output.error,
  }))
  if (outputs.length === 0) return { ...IDLE_PLAYBACK, outputs }

  const clocks = outputs
    .filter((output) => output.outputStatus !== 'missing' && output.playback)
    .map((output) => output.playback as AvProjectionPlaybackAck)

  let playingCount = 0
  let pausedCount = 0
  let endedCount = 0
  let errorCount = 0
  let loadingCount = 0
  for (const ack of clocks) {
    if (ack.status === 'playing') playingCount += 1
    else if (ack.status === 'paused') pausedCount += 1
    else if (ack.status === 'ended') endedCount += 1
    else if (ack.status === 'error') errorCount += 1
    else loadingCount += 1
  }
  for (const output of outputs) {
    if (output.outputStatus === 'failed' && !output.playback) errorCount += 1
  }

  const status = aggregatedPlaybackStatus({
    clocksLength: clocks.length,
    playingCount,
    pausedCount,
    endedCount,
    errorCount,
    loadingCount,
  })

  const clock =
    clocks.find((ack) => ack.status === 'playing') ??
    clocks.find((ack) => ack.status === 'paused') ??
    clocks.find((ack) => ack.status === 'ended') ??
    clocks[0]

  return {
    status,
    currentTimeMs: clock?.currentTimeMs ?? 0,
    durationMs: clock?.durationMs ?? null,
    seekableStartMs: clock?.seekableStartMs ?? 0,
    seekableEndMs: clock?.seekableEndMs ?? clock?.durationMs ?? 0,
    volume: clock?.volume ?? DEFAULT_AV_PLAYBACK_VOLUME,
    muted: clock?.muted ?? DEFAULT_AV_PLAYBACK_MUTED,
    loop: clock?.loop ?? DEFAULT_AV_PLAYBACK_LOOP,
    playingCount,
    pausedCount,
    endedCount,
    errorCount,
    loadingCount,
    outputs,
  }
}
