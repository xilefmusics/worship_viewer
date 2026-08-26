import { describe, expect, it } from 'vitest'

import type { components } from '@/api/schema'
import { playerFromReadyMedia } from '@/routes/player/media.$mediaId'

function media(status: components['schemas']['MediaStatus'], content: components['schemas']['MediaContent'] | null) {
  return {
    id: 'media:1',
    owner: 'team:1',
    title: 'Welcome video',
    status,
    content,
  } satisfies components['schemas']['Media']
}

describe('direct media player mapping', () => {
  it('creates one titled AV item for Ready media', () => {
    const player = playerFromReadyMedia(media('ready', { type: 'web_page', url: 'https://example.com' }))
    expect(player?.items).toEqual([{ type: 'media', id: 'media:1', title: 'Welcome video', content: { type: 'web_page', url: 'https://example.com' } }])
    expect(player?.toc).toEqual([{ idx: 0, id: 'media:1', title: 'Welcome video', nr: '', liked: false }])
  })

  it('maps Ready slide decks onto the same player item shape used for projection', () => {
    const player = playerFromReadyMedia(
      media('ready', { type: 'slide_deck', pages: [{ blob_id: 'p1' }, { blob_id: 'p2' }] }),
    )
    expect(player?.items[0]).toEqual({
      type: 'media',
      id: 'media:1',
      title: 'Welcome video',
      content: { type: 'slide_deck', pages: [{ blob_id: 'p1' }, { blob_id: 'p2' }] },
    })
  })

  it.each(['processing', 'failed'] as const)('rejects %s media', (status) => {
    expect(playerFromReadyMedia(media(status, null))).toBeNull()
  })

  it('rejects Ready media without playable content', () => {
    expect(playerFromReadyMedia(media('ready', null))).toBeNull()
  })
})
