import { describe, expect, it, vi } from 'vitest'

import {
  CHORD_SONG_FONT_SCALE_CHANGE_EVENT,
  CHORD_SONG_FONT_SCALE_STORAGE_KEY,
  clampChordSongFontScale,
  readChordSongFontScale,
  writeChordSongFontScale,
} from '@/lib/player/chord-song-font-scale-preference'

describe('chord song font scale preference', () => {
  it('defaults to 1 and rejects malformed stored values', () => {
    expect(readChordSongFontScale({ getItem: () => null })).toBe(1)
    expect(readChordSongFontScale({ getItem: () => 'not-a-number' })).toBe(1)
  })

  it('clamps values to 0.5–2', () => {
    expect(clampChordSongFontScale(0.1)).toBe(0.5)
    expect(clampChordSongFontScale(2.5)).toBe(2)
    expect(readChordSongFontScale({ getItem: () => '3' })).toBe(2)
  })

  it('persists and broadcasts the normalized value', () => {
    const storage = { setItem: vi.fn() }
    const dispatchEvent = vi.fn()
    vi.stubGlobal('window', { dispatchEvent })

    expect(writeChordSongFontScale(1.234, storage)).toBe(1.23)
    expect(storage.setItem).toHaveBeenCalledWith(CHORD_SONG_FONT_SCALE_STORAGE_KEY, '1.23')
    expect(dispatchEvent).toHaveBeenCalledWith(
      expect.objectContaining({ type: CHORD_SONG_FONT_SCALE_CHANGE_EVENT, detail: 1.23 }),
    )

    vi.unstubAllGlobals()
  })
})
