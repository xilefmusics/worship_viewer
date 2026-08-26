import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import type { Media } from '@/api/media'
import { MediaEditorScreen } from '@/components/media/MediaEditorScreen'

const fetchMedia = vi.fn()
const updateMedia = vi.fn()
const commitDeck = vi.fn()

vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}))
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }))
vi.mock('@/hooks/use-online', () => ({ useOnline: () => true }))
vi.mock('@/hooks/useTeamDetail', () => ({
  useTeamDetail: () => ({
    data: {
      id: 'team:1',
      name: 'Team',
      members: [{ user: { id: 'user:1' }, role: 'admin' }],
    },
  }),
}))
vi.mock('@/hooks/useWritableTeams', () => ({
  useWritableTeams: () => ({ user: { id: 'user:1' }, teams: [] }),
}))
vi.mock('@/api/media-upload', () => ({
  uploadMediaSource: vi.fn(),
  mediaAssetDataUrl: () => '/asset',
}))
vi.mock('@/api/media', () => ({
  fetchMedia: (...args: unknown[]) => fetchMedia(...args),
  updateMedia: (...args: unknown[]) => updateMedia(...args),
  commitDeck: (...args: unknown[]) => commitDeck(...args),
  beginDeckRevision: vi.fn(),
  cancelMediaProcessing: vi.fn(),
  mediaDetailKey: (id: string) => ['media', 'detail', id],
  mediaListRootKey: ['media'],
}))
vi.mock('@/components/media/DeckPagesEditor', () => ({
  DeckPagesEditor: ({ pages }: { pages: { id: string }[] }) => (
    <div data-testid="deck-pages">{pages.map((page) => page.id).join(',')}</div>
  ),
}))

function deckMedia(overrides: Partial<Media> = {}): Media {
  return {
    id: 'media:deck',
    owner: 'team:1',
    title: 'Sunday',
    status: 'processing',
    declared_kind: 'slide_deck',
    pending_revision: {
      operation: 'rev1',
      status: 'ready',
      pages: [
        { id: 'p1', blob_id: 'b1' },
        { id: 'p2', blob_id: 'b2' },
      ],
    },
    ...overrides,
  }
}

function renderEditor() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={client}>
      <MediaEditorScreen mediaId="media:deck" />
    </QueryClientProvider>,
  )
}

beforeEach(() => {
  vi.clearAllMocks()
  fetchMedia.mockResolvedValue(deckMedia())
  updateMedia.mockImplementation(async (_qc: unknown, _id: string, body: { title: string }) =>
    deckMedia({ title: body.title }),
  )
  commitDeck.mockResolvedValue(deckMedia({
    status: 'ready',
    pending_revision: undefined,
    content: { type: 'slide_deck', pages: [{ blob_id: 'b1' }, { blob_id: 'b2' }] },
  }))
})

describe('MediaEditorScreen slide decks', () => {
  it('M2: previews draft pages once expansion is idle', async () => {
    renderEditor()
    expect(await screen.findByTestId('deck-pages')).toHaveTextContent('p1,p2')
    expect(screen.getByText(/media.states.processing/)).toBeInTheDocument()
  })

  it('M4: save commits the draft page order', async () => {
    const user = userEvent.setup()
    renderEditor()
    await screen.findByTestId('deck-pages')
    await user.click(screen.getByRole('button', { name: 'media.actions.save' }))
    await waitFor(() => {
      expect(commitDeck).toHaveBeenCalledWith(expect.anything(), 'media:deck', {
        operation: 'rev1',
        page_ids: ['p1', 'p2'],
      })
    })
  })

  it('M5: disables save when the deck has no pages', async () => {
    fetchMedia.mockResolvedValue(deckMedia({
      pending_revision: { operation: 'rev1', status: 'ready', pages: [] },
    }))
    renderEditor()
    const save = await screen.findByRole('button', { name: 'media.actions.save' })
    expect(save).toBeDisabled()
  })

  it('M6: keeps the Ready deck visible after a replacement failure', async () => {
    fetchMedia.mockResolvedValue(deckMedia({
      status: 'ready',
      declared_kind: undefined,
      content: { type: 'slide_deck', pages: [{ blob_id: 'live' }] },
      pending_revision: {
        operation: 'rev2',
        status: 'failed',
        processing_error: { code: 'media_input_invalid', detail: 'Encrypted PDF.' },
        pages: [{ id: 'p1', blob_id: 'live' }],
      },
    }))
    renderEditor()
    expect(await screen.findByText('media.editor.replacementFailedTitle')).toBeInTheDocument()
    expect(screen.getByText('Encrypted PDF.')).toBeInTheDocument()
    expect(screen.getByTestId('deck-pages')).toHaveTextContent('p1')
  })
})
