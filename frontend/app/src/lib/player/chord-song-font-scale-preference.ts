import { getLocalStorage, safeGetItem, safeSetItem } from '@/lib/browser-storage'

export const CHORD_SONG_FONT_SCALE_MIN = 0.5
export const CHORD_SONG_FONT_SCALE_MAX = 2
export const DEFAULT_CHORD_SONG_FONT_SCALE = 1
export const CHORD_SONG_FONT_SCALE_STORAGE_KEY = 'wv_chord_song_font_scale'
export const CHORD_SONG_FONT_SCALE_CHANGE_EVENT = 'wv-chord-song-font-scale-change'

export function clampChordSongFontScale(value: number): number {
  if (!Number.isFinite(value)) return DEFAULT_CHORD_SONG_FONT_SCALE
  const clamped = Math.min(CHORD_SONG_FONT_SCALE_MAX, Math.max(CHORD_SONG_FONT_SCALE_MIN, value))
  return Math.round(clamped * 100) / 100
}

export function readChordSongFontScale(
  storage: Pick<Storage, 'getItem'> | null = getLocalStorage(),
): number {
  const raw = safeGetItem(CHORD_SONG_FONT_SCALE_STORAGE_KEY, storage)
  if (raw == null || raw.trim() === '') return DEFAULT_CHORD_SONG_FONT_SCALE
  const parsed = Number(raw)
  return Number.isFinite(parsed)
    ? clampChordSongFontScale(parsed)
    : DEFAULT_CHORD_SONG_FONT_SCALE
}

export function writeChordSongFontScale(
  value: number,
  storage: Pick<Storage, 'setItem'> | null = getLocalStorage(),
): number {
  const normalized = clampChordSongFontScale(value)
  safeSetItem(CHORD_SONG_FONT_SCALE_STORAGE_KEY, String(normalized), storage)

  if (typeof globalThis.window !== 'undefined') {
    globalThis.window.dispatchEvent(
      new CustomEvent(CHORD_SONG_FONT_SCALE_CHANGE_EVENT, { detail: normalized }),
    )
  }

  return normalized
}
