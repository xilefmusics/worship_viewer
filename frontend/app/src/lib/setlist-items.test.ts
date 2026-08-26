import { describe, expect, it } from 'vitest'

import {
  countSetlistItems,
  mergeEditorSongsIntoItems,
  songLinksFromSetlistItems,
} from '@/lib/setlist-items'

describe('setlist items', () => {
  it('extracts song links in persisted order', () => {
    expect(
      songLinksFromSetlistItems([
        { type: 'media', id: 'm1' },
        { type: 'song', id: 's1', nr: '1', key: null, tempo: null, language: null, flow: null },
        { type: 'media', id: 'm2' },
        { type: 'song', id: 's2', nr: null, key: null, tempo: 88, language: 'de', flow: null },
      ]).map((link) => link.id),
    ).toEqual(['s1', 's2'])
  })

  it.each([
    ['empty', [], { songs: 0, media: 0 }],
    [
      'songs only',
      [
        { type: 'song' as const, id: 's1', nr: null, key: null, tempo: null, language: null, flow: null },
        { type: 'song' as const, id: 's2', nr: null, key: null, tempo: null, language: null, flow: null },
      ],
      { songs: 2, media: 0 },
    ],
    ['media only', [{ type: 'media' as const, id: 'm1' }], { songs: 0, media: 1 }],
    [
      'mixed items with a repeated media id',
      [
        { type: 'media' as const, id: 'm1' },
        { type: 'song' as const, id: 's1', nr: null, key: null, tempo: null, language: null, flow: null },
        { type: 'media' as const, id: 'm1' },
      ],
      { songs: 1, media: 2 },
    ],
  ])('counts %s', (_label, items, expected) => {
    expect(countSetlistItems(items)).toEqual(expected)
  })

  it('treats an absent items array as empty', () => {
    expect(countSetlistItems(undefined)).toEqual({ songs: 0, media: 0 })
    expect(countSetlistItems(null)).toEqual({ songs: 0, media: 0 })
  })

  it('preserves media slots when the songs-only editor rewrites songs', () => {
    const original = [
      { type: 'media' as const, id: 'm1' },
      { type: 'song' as const, id: 's1', nr: '1', key: null, tempo: null, language: null, flow: null },
      { type: 'media' as const, id: 'm2' },
    ]
    expect(
      mergeEditorSongsIntoItems(original, [{ id: 's2', key: null, nr: '3', flow: null }]),
    ).toEqual([
      { type: 'media', id: 'm1' },
      { type: 'song', id: 's2', nr: '3', key: null, tempo: null, language: null, flow: null },
      { type: 'media', id: 'm2' },
    ])
  })
})
