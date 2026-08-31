import { QueryClient } from '@tanstack/react-query'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import type { components } from '@/api/schema'
import { setSongLikeStatus } from '@/api/songs-like'
import { playerQueryKey, playerResourceTitleKey, songDetailQueryKey } from '@/lib/setlist-detail-key'

const mocks = vi.hoisted(() => ({
  deleteLike: vi.fn(),
  putLike: vi.fn(),
}))

vi.mock('@/api/client', () => ({
  api: {
    DELETE: mocks.deleteLike,
    PUT: mocks.putLike,
  },
}))

vi.mock('@/lib/api-unauthorized', () => ({
  redirectToLoginAfterUnauthorized: vi.fn(),
}))

type Player = components['schemas']['Player']
type Song = components['schemas']['Song']

function song(id: string, liked: boolean): Song {
  return {
    id,
    blobs: [],
    not_a_song: false,
    owner: 'team-1',
    user_specific_addons: { liked },
    data: { titles: [id], sections: [] },
  } as Song
}

function player(): Player {
  return {
    index: 0,
    between_items: false,
    orientation: 'portrait',
    scroll_type: 'book',
    scroll_type_cache_other_orientation: 'book',
    toc: [
      { idx: 0, nr: '1', title: 'First', id: 'song-1', liked: true },
      { idx: 1, nr: '2', title: 'Duplicate', id: 'song-1', liked: true },
    ],
    items: [
      { type: 'chords', song: song('song-1', true), language: null, flow: null },
      { type: 'chords', song: song('song-1', true), language: null, flow: null },
    ],
  }
}

beforeEach(() => {
  vi.clearAllMocks()
  mocks.deleteLike.mockResolvedValue({
    response: { ok: true, status: 204 },
    error: undefined,
  })
})

describe('setSongLikeStatus', () => {
  it('reconciles every cached player occurrence and the song detail after an unlike', async () => {
    const queryClient = new QueryClient()
    queryClient.setQueryData(playerQueryKey('setlist', 'setlist-1', 'book'), {
      status: 'ready',
      source: 'network',
      player: player(),
    })
    queryClient.setQueryData(songDetailQueryKey('song-1'), song('song-1', true))
    queryClient.setQueryData(playerResourceTitleKey('setlist', 'setlist-1'), 'Sunday')

    await setSongLikeStatus(queryClient, { id: 'song-1', liked: false })

    const cached = queryClient.getQueryData<{
      status: 'ready'
      player: Player
    }>(playerQueryKey('setlist', 'setlist-1', 'book'))
    expect(cached?.player.toc.every((row) => !row.liked)).toBe(true)
    expect(
      cached?.player.items.every(
        (item) => item.type !== 'chords' || !item.song.user_specific_addons.liked,
      ),
    ).toBe(true)
    expect(queryClient.getQueryData<Song>(songDetailQueryKey('song-1'))?.user_specific_addons.liked).toBe(false)
    expect(queryClient.getQueryData(playerResourceTitleKey('setlist', 'setlist-1'))).toBe('Sunday')
  })
})
