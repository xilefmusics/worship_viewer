import {
  sameAvProjectionContent,
  type AvProjectionAck,
  type AvProjectionAckError,
  type AvProjectionCommand,
  type AvProjectionCommandId,
  type AvProjectionContent,
  type AvProjectionMessage,
  type AvProjectionPresence,
} from '@/lib/player/av-projection-protocol'

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
    const outputs: Record<string, AvTrackedOutput> = {}
    for (const [id, output] of Object.entries(state.outputs)) {
      outputs[id] = refreshOutputStatus(
        { ...output, error: undefined, lastAckCommandId: output.lastAckCommandId },
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
  }
}
