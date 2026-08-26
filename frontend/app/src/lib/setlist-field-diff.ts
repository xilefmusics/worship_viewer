import type { components } from '@/api/schema'

import { mergeEditorSongsIntoItems, setlistItemsEqual, type SetlistItem } from '@/lib/setlist-items'
import type { EditorSongLink } from '@/lib/setlist-song-links'

export type Setlist = components['schemas']['Setlist']
export type SetlistPatchDirty = components['schemas']['PatchSetlist']

export type SetlistDiffBaseline = {
  title: string
  items: SetlistItem[]
  songs?: EditorSongLink[]
  owner: string
}

/**
 * PATCH body with only dirty top-level fields; `null` when nothing to send.
 */
export function buildSetlistPatchBody(
  baseline: SetlistDiffBaseline,
  draft: { title: string; items?: SetlistItem[]; songs?: EditorSongLink[]; owner: string },
): SetlistPatchDirty | null {
  const body: SetlistPatchDirty = {}
  if (draft.title !== baseline.title) {
    body.title = draft.title
  }
  const draftOwner = draft.owner.trim()
  if (draftOwner && draftOwner !== baseline.owner) {
    body.owner = draftOwner
  }
  const nextItems = draft.songs
    ? mergeEditorSongsIntoItems(baseline.items, draft.songs)
    : draft.items ?? []
  if (!setlistItemsEqual(baseline.items, nextItems)) {
    body.items = nextItems
  }
  if (body.title === undefined && body.items === undefined && body.owner === undefined) return null
  return body
}
