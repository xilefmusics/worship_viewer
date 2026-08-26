import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { MediaListView } from '@/components/media/MediaListView'

const refetch = vi.fn()
const setQInput = vi.fn()
let queryResult: Record<string, unknown>

vi.mock('@tanstack/react-query', () => ({
  useQueryClient: () => ({ invalidateQueries: vi.fn() }),
  useInfiniteQuery: () => queryResult,
  useMutation: () => ({ isPending: false, mutate: vi.fn() }),
}))
vi.mock('@tanstack/react-router', () => ({ useNavigate: () => vi.fn() }))
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, options?: Record<string, string>) =>
      key === 'media.row.openAria' && options
        ? `${options.title} ${options.kind} ${options.status}`
        : key,
  }),
}))
vi.mock('@/hooks/useHubSearch', () => ({ useHubSearch: () => ({ debouncedQ: '', selectedTeamId: null, setQInput }) }))
vi.mock('@/hooks/useSession', () => ({ useSession: () => ({ data: { id: 'user:guest' } }) }))
vi.mock('@/hooks/useTeamDetail', () => ({ useTeamDetail: () => ({ data: { id: 'team:1', name: 'Team', members: [{ user: { id: 'user:guest' }, role: 'guest' }] } }) }))
vi.mock('@/hooks/useWritableTeams', () => ({ useWritableTeams: () => ({ teams: [], user: { id: 'user:guest' } }) }))

beforeEach(() => {
  refetch.mockReset()
  queryResult = { data: { pages: [{ items: [], total: 0 }] }, isPending: false, isError: false, isRefetching: false, refetch, hasNextPage: false }
})

describe('MediaListView', () => {
  it('renders Ready, Processing, Failed, and unknown-safe rows accessibly without read-only actions', () => {
    queryResult.data = { pages: [{ total: 4, items: [
      { id: '1', owner: 'team:1', title: 'YouTube', status: 'ready', content: { type: 'youtube', video_id: 'abcdefghijk', canonical_url: 'https://www.youtube.com/watch?v=abcdefghijk' } },
      { id: '2', owner: 'team:1', title: 'Pending', status: 'processing' },
      { id: '3', owner: 'team:1', title: 'Broken', status: 'failed' },
      { id: '4', owner: 'team:1', title: 'Future', status: 'future', content: { type: 'future' } },
    ] }] }
    render(<MediaListView />)
    expect(screen.getByRole('button', { name: /YouTube.*media.states.ready/ })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Pending.*media.states.processing/ })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Broken.*media.states.failed/ })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Future.*media.states.unknown/ })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /Actions for/ })).not.toBeInTheDocument()
  })

  it('renders empty, error/retry, and refresh states', async () => {
    const user = userEvent.setup()
    const { rerender } = render(<MediaListView />)
    expect(screen.getByText('media.list.empty')).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'media.actions.refresh' }))
    expect(refetch).toHaveBeenCalled()
    queryResult = { ...queryResult, isError: true }
    rerender(<MediaListView />)
    expect(screen.getByText('media.list.error')).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'hub.error.retry' }))
    expect(refetch).toHaveBeenCalledTimes(2)
  })
})
