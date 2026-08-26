import { getLocalStorage, safeGetItem, safeSetItem } from '@/lib/browser-storage'
import {
  parseAvProjectionCommandSnapshot,
  parseAvProjectionMessage,
  type AvProjectionCommand,
  type AvProjectionMessage,
} from '@/lib/player/av-projection-protocol'

export const AV_PROJECTION_STORAGE_PREFIX = 'wvAvProjection:'
export const AV_OUTPUT_ID_STORAGE_KEY = 'wvAvOutputId'

/** Single projection channel per browser profile — all AV players share one output. */
export const AV_PROJECTION_SHARED_SESSION_ID = 'shared'

const STORAGE_DEBOUNCE_MS = 75

function storageKey(sessionId: string): string {
  return `${AV_PROJECTION_STORAGE_PREFIX}${sessionId}`
}

function channelName(sessionId: string): string {
  return `wv-av-${sessionId}`
}

export type AvProjectionChannel = {
  send: (message: AvProjectionMessage) => void
  readLatestCommand: () => AvProjectionCommand | null
  close: () => void
}

export function readAvProjectionSnapshot(
  sessionId: string,
  storage: Pick<Storage, 'getItem'> | null = getLocalStorage(),
): AvProjectionCommand | null {
  try {
    const raw = safeGetItem(storageKey(sessionId), storage)
    if (!raw) return null
    return parseAvProjectionCommandSnapshot(JSON.parse(raw), sessionId)
  } catch {
    return null
  }
}

export function writeAvProjectionSnapshot(
  sessionId: string,
  command: AvProjectionCommand,
  storage: Pick<Storage, 'setItem'> | null = getLocalStorage(),
): void {
  safeSetItem(storageKey(sessionId), JSON.stringify(command), storage)
}

function persistIfCommand(
  sessionId: string,
  message: AvProjectionMessage,
  storage: Pick<Storage, 'getItem' | 'setItem'> | null,
  pending: { current: AvProjectionCommand | null },
  timer: { current: ReturnType<typeof setTimeout> | null },
): void {
  if (message.type !== 'command') return
  pending.current = message
  if (!storage) return
  if (timer.current != null) clearTimeout(timer.current)
  timer.current = setTimeout(() => {
    timer.current = null
    if (!pending.current) return
    writeAvProjectionSnapshot(sessionId, pending.current, storage)
    pending.current = null
  }, STORAGE_DEBOUNCE_MS)
}

export function createAvProjectionChannel(
  sessionId: string,
  onMessage: (message: AvProjectionMessage) => void,
  storage: Pick<Storage, 'getItem' | 'setItem'> | null = getLocalStorage(),
): AvProjectionChannel {
  let channel: BroadcastChannel | null = null
  const pending: { current: AvProjectionCommand | null } = { current: null }
  const timer: { current: ReturnType<typeof setTimeout> | null } = { current: null }
  try {
    channel = new BroadcastChannel(channelName(sessionId))
  } catch {
    channel = null
  }

  const flushStorage = () => {
    if (timer.current != null) {
      clearTimeout(timer.current)
      timer.current = null
    }
    if (pending.current) {
      writeAvProjectionSnapshot(sessionId, pending.current, storage)
      pending.current = null
    }
  }

  if (channel) {
    channel.onmessage = (event: MessageEvent<unknown>) => {
      const parsed = parseAvProjectionMessage(event.data)
      if (parsed) onMessage(parsed)
    }
  }

  const onStorage = (event: StorageEvent) => {
    if (event.key !== storageKey(sessionId) || !event.newValue) return
    try {
      const command = parseAvProjectionCommandSnapshot(JSON.parse(event.newValue), sessionId)
      if (command) onMessage(command)
    } catch {
      /* ignore malformed payload */
    }
  }

  if (typeof window !== 'undefined') {
    window.addEventListener('storage', onStorage)
  }

  return {
    send(message) {
      channel?.postMessage(message)
      persistIfCommand(sessionId, message, storage, pending, timer)
    },
    readLatestCommand() {
      return readAvProjectionSnapshot(sessionId, storage)
    },
    close() {
      flushStorage()
      channel?.close()
      channel = null
      if (typeof window !== 'undefined') {
        window.removeEventListener('storage', onStorage)
      }
    },
  }
}

/** @deprecated Prefer {@link createAvProjectionChannel}. Controller-only send wrapper. */
export type AvProjectionSync = {
  broadcast: (command: AvProjectionCommand) => void
  readLatest: () => AvProjectionCommand | null
  close: () => void
}

export function createAvProjectionSync(
  sessionId: string,
  storage: Pick<Storage, 'getItem' | 'setItem'> | null = getLocalStorage(),
): AvProjectionSync {
  const channel = createAvProjectionChannel(sessionId, () => {}, storage)
  return {
    broadcast: (command) => channel.send(command),
    readLatest: () => channel.readLatestCommand(),
    close: () => channel.close(),
  }
}

export type AvProjectionListener = {
  close: () => void
}

export function subscribeAvProjectionSync(
  sessionId: string,
  onCommand: (command: AvProjectionCommand) => void,
  storage: Pick<Storage, 'getItem'> | null = getLocalStorage(),
): AvProjectionListener {
  const latest = readAvProjectionSnapshot(sessionId, storage)
  if (latest) onCommand(latest)

  const channel = createAvProjectionChannel(sessionId, (message) => {
    if (message.type === 'command') onCommand(message)
  }, storage as Pick<Storage, 'getItem' | 'setItem'> | null)

  return { close: () => channel.close() }
}

export function getAvProjectionSessionId(): string {
  return AV_PROJECTION_SHARED_SESSION_ID
}

/** @deprecated All players use {@link getAvProjectionSessionId}. */
export function createAvProjectionSessionId(): string {
  return getAvProjectionSessionId()
}

export function newAvOutputWindowName(): string {
  const id =
    typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function'
      ? crypto.randomUUID()
      : `${Date.now()}-${Math.random().toString(16).slice(2)}`
  return `wv-av-output-${id}`
}

export function readOrCreateAvOutputId(
  storage: Pick<Storage, 'getItem' | 'setItem'> | null = getSessionStorage(),
): string {
  const existing = storage ? safeGetItem(AV_OUTPUT_ID_STORAGE_KEY, storage) : null
  if (existing && existing.trim()) return existing
  const created =
    typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function'
      ? crypto.randomUUID()
      : `output-${Date.now()}`
  if (storage) safeSetItem(AV_OUTPUT_ID_STORAGE_KEY, created, storage)
  return created
}

function getSessionStorage(): Pick<Storage, 'getItem' | 'setItem'> | null {
  try {
    return typeof globalThis.sessionStorage === 'undefined' ? null : globalThis.sessionStorage
  } catch {
    return null
  }
}
