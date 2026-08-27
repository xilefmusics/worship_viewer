import { describe, expect, it } from 'vitest'

import type { components } from '@/api/schema'
import { playerFromReadyMedia } from '@/routes/player/media.$mediaId'

function media(content: components['schemas']['MediaContent']): components['schemas']['Media'] {
  return { id: 'media:1', owner: 'team:1', title: 'Welcome video', content }
}

describe('direct media player mapping', () => {
  it('creates one titled AV item for stored media', () => {
    const player = playerFromReadyMedia(media({ type: 'web_page', url: 'https://example.com' }))
    expect(player?.items).toEqual([{ type: 'media', id: 'media:1', title: 'Welcome video', content: { type: 'web_page', url: 'https://example.com' } }])
    expect(player?.toc).toEqual([{ idx: 0, id: 'media:1', title: 'Welcome video', nr: '', liked: false }])
  })

  it('maps slide decks onto the same player item shape used for projection', () => {
    const player = playerFromReadyMedia(media({ type: 'slide_deck', pages: [{ blob_id: 'p1' }, { blob_id: 'p2' }] }))
    expect(player?.items[0]).toEqual({
      type: 'media',
      id: 'media:1',
      title: 'Welcome video',
      content: { type: 'slide_deck', pages: [{ blob_id: 'p1' }, { blob_id: 'p2' }] },
    })
  })
})
