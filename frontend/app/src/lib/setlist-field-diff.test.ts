import { describe, expect, it } from 'vitest'

import { buildSetlistPatchBody } from '@/lib/setlist-field-diff'
import { mergeEditorSongsIntoItems } from '@/lib/setlist-items'
import type { EditorSongLink } from '@/lib/setlist-song-links'

const ownerA = 'team-a'

function withItems(songs: EditorSongLink[]) {
  return {
    title: 'A',
    owner: ownerA,
    songs,
    items: mergeEditorSongsIntoItems([], songs),
  }
}

const songC: EditorSongLink = { id: 'x', key: 'C', nr: '1', flow: null }

describe('buildSetlistPatchBody', () => {
  it('returns null when draft matches normalized baseline', () => {
    expect(
      buildSetlistPatchBody(withItems([songC]), {
        title: 'A',
        owner: ownerA,
        songs: [{ id: 'x', key: 'C', nr: '1', flow: null }],
      }),
    ).toBeNull()
  })

  it('sends title when changed', () => {
    expect(
      buildSetlistPatchBody(withItems([songC]), {
        title: 'B',
        owner: ownerA,
        songs: [{ id: 'x', key: 'C', nr: '1', flow: null }],
      }),
    ).toEqual({ title: 'B' })
  })

  it('sends items when order differs', () => {
    expect(
      buildSetlistPatchBody(withItems([songC]), {
        title: 'A',
        owner: ownerA,
        songs: [
          { id: 'y', key: null, nr: null, flow: null },
          { id: 'x', key: 'C', nr: '1', flow: null },
        ],
      }),
    ).toEqual({
      items: [
        { type: 'song', id: 'y', nr: null, key: null, tempo: null, language: null, flow: null },
        { type: 'song', id: 'x', nr: '1', key: { level: 3 }, tempo: null, language: null, flow: null },
      ],
    })
  })

  it('treats equivalent chord symbols as unchanged', () => {
    expect(
      buildSetlistPatchBody(withItems([songC]), {
        title: 'A',
        owner: ownerA,
        songs: [{ id: 'x', key: 'C', nr: '1', flow: null }],
      }),
    ).toBeNull()
  })

  it('detects slot key drift', () => {
    expect(
      buildSetlistPatchBody(withItems([songC]), {
        title: 'A',
        owner: ownerA,
        songs: [{ id: 'x', key: null, nr: '1', flow: null }],
      }),
    ).toEqual({
      items: [{ type: 'song', id: 'x', nr: '1', key: null, tempo: null, language: null, flow: null }],
    })
  })

  it('serializes slot keys to pitch-class `{ level }` objects for PATCH', () => {
    expect(
      buildSetlistPatchBody(withItems([songC]), {
        title: 'A',
        owner: ownerA,
        songs: [{ id: 'x', key: 'F', nr: '1', flow: null }],
      }),
    ).toEqual({
      items: [{ type: 'song', id: 'x', nr: '1', key: { level: 8 }, tempo: null, language: null, flow: null }],
    })
  })

  it('stringifies numeric song ids for PATCH bodies', () => {
    const numBase = withItems([{ id: '7', key: null }])
    expect(
      buildSetlistPatchBody(numBase, {
        title: 'A',
        owner: ownerA,
        songs: [{ id: '7', key: 'C', flow: null }],
      }),
    ).toEqual({
      items: [{ type: 'song', id: '7', nr: null, key: { level: 3 }, tempo: null, language: null, flow: null }],
    })
  })

  it('sends owner when changed', () => {
    expect(
      buildSetlistPatchBody(withItems([songC]), {
        title: 'A',
        owner: 'team-b',
        songs: [{ id: 'x', key: 'C', nr: '1', flow: null }],
      }),
    ).toEqual({ owner: 'team-b' })
  })

  it('omits empty owner drafts', () => {
    expect(
      buildSetlistPatchBody(withItems([songC]), {
        title: 'A',
        owner: '',
        songs: [{ id: 'x', key: 'C', nr: '1', flow: null }],
      }),
    ).toBeNull()
  })

  it('detects slot tempo drift', () => {
    expect(
      buildSetlistPatchBody(withItems([{ ...songC, tempo: 88 }]), {
        title: 'A',
        owner: ownerA,
        songs: [{ id: 'x', key: 'C', nr: '1', tempo: 88, flow: null }],
      }),
    ).toBeNull()
    expect(
      buildSetlistPatchBody(withItems([songC]), {
        title: 'A',
        owner: ownerA,
        songs: [{ id: 'x', key: 'C', nr: '1', tempo: 88, flow: null }],
      }),
    ).toEqual({
      items: [{ type: 'song', id: 'x', nr: '1', key: { level: 3 }, tempo: 88, language: null, flow: null }],
    })
  })

  it('detects slot language drift', () => {
    expect(
      buildSetlistPatchBody(withItems([songC]), {
        title: 'A',
        owner: ownerA,
        songs: [{ id: 'x', key: 'C', nr: '1', language: 'de', flow: null }],
      }),
    ).toEqual({
      items: [{ type: 'song', id: 'x', nr: '1', key: { level: 3 }, tempo: null, language: 'de', flow: null }],
    })
  })

  it('treats matching language overrides as unchanged', () => {
    expect(
      buildSetlistPatchBody(withItems([{ ...songC, language: 'de' }]), {
        title: 'A',
        owner: ownerA,
        songs: [{ id: 'x', key: 'C', nr: '1', language: ' de ', flow: null }],
      }),
    ).toBeNull()
  })

  it('preserves media slots when only songs change', () => {
    const items = [
      { type: 'media' as const, id: 'm1' },
      { type: 'song' as const, id: 'x', nr: '1', key: { level: 3 }, tempo: null, language: null, flow: null },
    ]
    expect(
      buildSetlistPatchBody(
        { title: 'A', owner: ownerA, items, songs: [songC] },
        { title: 'A', owner: ownerA, songs: [{ id: 'y', key: null, nr: null, flow: null }] },
      ),
    ).toEqual({
      items: [
        { type: 'media', id: 'm1' },
        { type: 'song', id: 'y', nr: null, key: null, tempo: null, language: null, flow: null },
      ],
    })
  })
})
