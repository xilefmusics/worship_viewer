import { describe, expect, it, vi } from 'vitest'

import { DEFAULT_AV_PREFERENCES } from '@/lib/player/av-preferences'
import { buildAvProjectionCommand, type AvProjectionCommand } from '@/lib/player/av-projection-protocol'
import {
  AV_OUTPUT_ID_STORAGE_KEY,
  AV_PROJECTION_SHARED_SESSION_ID,
  createAvProjectionChannel,
  createAvProjectionSessionId,
  createAvProjectionSync,
  getAvProjectionSessionId,
  newAvOutputWindowName,
  readAvProjectionSnapshot,
  readOrCreateAvOutputId,
  writeAvProjectionSnapshot,
} from '@/lib/player/av-projection-sync'

const layers = {
  contentLayer: DEFAULT_AV_PREFERENCES.contentLayer,
  backgroundLayer: DEFAULT_AV_PREFERENCES.backgroundLayer,
  transition: DEFAULT_AV_PREFERENCES.transition,
}

function lyricsCommand(commandId: number, text: string): AvProjectionCommand {
  return buildAvProjectionCommand({
    sessionId: 'session-1',
    commandId,
    ...layers,
    screenState: 'live',
    itemTitle: 'Song',
    nextPreview: 'Next',
    content: { type: 'lyrics', contentText: text, contentLines: [{ primary: text }] },
  })
}

function memoryStorage() {
  const map = new Map<string, string>()
  return {
    getItem: (key: string) => map.get(key) ?? null,
    setItem: (key: string, value: string) => {
      map.set(key, value)
    },
  }
}

describe('av-projection-sync', () => {
  it('uses one shared session id for every AV player', () => {
    expect(getAvProjectionSessionId()).toBe(AV_PROJECTION_SHARED_SESSION_ID)
    expect(createAvProjectionSessionId()).toBe(AV_PROJECTION_SHARED_SESSION_ID)
  })

  it('writes and reads tagged command snapshots', () => {
    const storage = memoryStorage()
    const command = lyricsCommand(1, 'Hello')
    writeAvProjectionSnapshot('session-1', command, storage)
    expect(readAvProjectionSnapshot('session-1', storage)).toEqual(command)
  })

  it('adapts legacy lyric snapshots into commands', () => {
    const storage = memoryStorage()
    storage.setItem(
      'wvAvProjection:session-legacy',
      JSON.stringify({
        contentText: 'Hello',
        contentLines: [{ primary: 'Hello', secondary: 'Hallo' }],
        ...layers,
        screenState: 'live',
        itemTitle: 'Song',
        nextPreview: null,
      }),
    )
    expect(readAvProjectionSnapshot('session-legacy', storage)?.content).toEqual({
      type: 'lyrics',
      contentText: 'Hello',
      contentLines: [{ primary: 'Hello', secondary: 'Hallo' }],
    })
  })

  it('broadcast persists structured lyrics in the latest command snapshot', () => {
    const storage = memoryStorage()
    const sync = createAvProjectionSync('session-bilingual-broadcast', storage)
    const command = lyricsCommand(4, 'Hallo')
    sync.broadcast(command)
    sync.close()
    expect(readAvProjectionSnapshot('session-bilingual-broadcast', storage)).toEqual(command)
  })

  it('does not persist acks or presence as the latest command', () => {
    const storage = memoryStorage()
    const received: unknown[] = []
    const channel = createAvProjectionChannel('session-ack', (message) => received.push(message), storage)
    channel.send({ type: 'hello', sessionId: 'session-ack', outputId: 'out-1', ready: true })
    channel.close()
    expect(readAvProjectionSnapshot('session-ack', storage)).toBeNull()
    expect(received).toEqual([])
  })

  it('debounces rapid localStorage writes', () => {
    vi.useFakeTimers()
    const setItem = vi.fn()
    const storage = {
      getItem: () => null,
      setItem,
    }
    const sync = createAvProjectionSync('session-3', storage)
    sync.broadcast(lyricsCommand(1, 'A'))
    sync.broadcast(lyricsCommand(2, 'B'))
    expect(setItem).not.toHaveBeenCalled()
    vi.advanceTimersByTime(75)
    expect(setItem).toHaveBeenCalledOnce()
    sync.close()
    vi.useRealTimers()
  })

  it('assigns a stable output id and unique window names', () => {
    const storage = memoryStorage()
    const first = readOrCreateAvOutputId(storage)
    expect(readOrCreateAvOutputId(storage)).toBe(first)
    expect(storage.getItem(AV_OUTPUT_ID_STORAGE_KEY)).toBe(first)
    expect(newAvOutputWindowName()).toMatch(/^wv-av-output-/)
    expect(newAvOutputWindowName()).not.toBe(newAvOutputWindowName())
  })
})
