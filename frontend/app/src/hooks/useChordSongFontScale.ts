import { useEffect, useState } from 'react'

import {
  CHORD_SONG_FONT_SCALE_CHANGE_EVENT,
  readChordSongFontScale,
} from '@/lib/player/chord-song-font-scale-preference'

export function useChordSongFontScale(): number {
  const [fontScale, setFontScale] = useState(readChordSongFontScale)

  useEffect(() => {
    const onChange = (event: Event) => {
      const detail = (event as CustomEvent<number>).detail
      setFontScale(detail ?? readChordSongFontScale())
    }

    globalThis.window.addEventListener(CHORD_SONG_FONT_SCALE_CHANGE_EVENT, onChange)
    return () => globalThis.window.removeEventListener(CHORD_SONG_FONT_SCALE_CHANGE_EVENT, onChange)
  }, [])

  return fontScale
}
