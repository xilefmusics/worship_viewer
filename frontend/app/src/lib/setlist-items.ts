import type { components } from '@/api/schema'

import { songLinkForSetlistMutation, type EditorSongLink, type SetlistSongLink } from '@/lib/setlist-song-links'

export type SetlistItem = components['schemas']['SetlistItem']
export type SetlistSongItem = Extract<SetlistItem, { type: 'song' }>
export type SetlistMediaItem = Extract<SetlistItem, { type: 'media' }>

export function isSetlistSongItem(item: SetlistItem): item is SetlistSongItem {
  return item.type === 'song'
}

export function songLinksFromSetlistItems(items: SetlistItem[] | null | undefined): SetlistSongLink[] {
  return (items ?? []).filter(isSetlistSongItem)
}

export function countSetlistItems(items: SetlistItem[] | null | undefined): {
  songs: number
  media: number
} {
  return (items ?? []).reduce(
    (counts, item) => {
      counts[item.type === 'song' ? 'songs' : 'media'] += 1
      return counts
    },
    { songs: 0, media: 0 },
  )
}

export function songItemFromLink(link: SetlistSongLink): SetlistSongItem {
  return { type: 'song', ...link }
}

export function mediaItemFromId(id: string): SetlistMediaItem {
  return { type: 'media', id }
}

/** Rebuild tagged `items` from the songs-only editor while preserving media slots. */
export function mergeEditorSongsIntoItems(
  original: SetlistItem[] | null | undefined,
  songs: EditorSongLink[],
): SetlistItem[] {
  const songItems = songs.map((link) => songItemFromLink(songLinkForSetlistMutation(link)))
  if (!(original ?? []).some((item) => item.type === 'media')) {
    return songItems
  }
  const queue = [...songItems]
  const next: SetlistItem[] = []
  for (const item of original ?? []) {
    if (item.type === 'media') {
      next.push(item)
      continue
    }
    const song = queue.shift()
    if (song) next.push(song)
  }
  next.push(...queue)
  return next
}

export function setlistItemsEqual(a: SetlistItem[], b: SetlistItem[]): boolean {
  if (a.length !== b.length) return false
  return JSON.stringify(a) === JSON.stringify(b)
}
