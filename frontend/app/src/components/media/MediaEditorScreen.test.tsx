import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import type { Media } from '@/api/media'
import { MediaEditorScreen } from '@/components/media/MediaEditorScreen'

const fetchMedia = vi.fn()
const updateMedia = vi.fn()
const commitDeck = vi.fn()
const uploadMediaSource = vi.fn()

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
  uploadMediaSource: (...args: unknown[]) => uploadMediaSource(...args),
  mediaAssetDataUrl: () => '/asset',
}))
vi.mock('@/api/media', () => ({
  fetchMedia: (...args: unknown[]) => fetchMedia(...args),
  updateMedia: (...args: unknown[]) => updateMedia(...args),
  commitDeck: (...args: unknown[]) => commitDeck(...args),
  beginDeckRevision: vi.fn(),
  mediaDetailKey: (id: string) => ['media', 'detail', id],
  mediaListRootKey: ['media'],
}))
vi.mock('@/components/media/DeckPagesEditor', () => ({
  DeckPagesEditor: ({
    pages,
    onAdd,
  }: {
    pages: { id: string }[]
    onAdd: (files: File[], insertionIndex: number) => void
  }) => (
    <div>
      <div data-testid="deck-pages">{pages.map((page) => page.id).join(',')}</div>
      <button type="button" onClick={() => onAdd([new File(['new'], 'new.png', { type: 'image/png' })], 1)}>
        add at boundary 1
      </button>
    </div>
  ),
}))

function deckMedia(overrides: Partial<Media> = {}): Media {
  return {
    id: 'media:deck',
    owner: 'team:1',
    title: 'Sunday',
    content: {
      type: 'slide_deck',
      pages: [
        { blob_id: 'b1', section_title: 'Section 1' },
        { blob_id: 'b2' },
      ],
    },
    pending_revision: {
      revision_id: 'rev1',
      pages: [
        { id: 'p1', blob_id: 'b1', section_title: 'Section 1' },
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
    pending_revision: undefined,
    content: {
      type: 'slide_deck',
      pages: [
        { blob_id: 'b1', section_title: 'Section 1' },
        { blob_id: 'b2' },
      ],
    },
  }))
})

describe('MediaEditorScreen slide decks', () => {
  it('M2: previews draft pages once expansion is idle', async () => {
    renderEditor()
    expect(await screen.findByTestId('deck-pages')).toHaveTextContent('p1,p2')
    expect(screen.queryByDisplayValue('Sunday')).not.toBeInTheDocument()
    expect(screen.queryByRole('link', { name: 'media.actions.play' })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'media.actions.save' })).not.toBeInTheDocument()
    expect(screen.queryByText('media.kinds.slide_deck')).not.toBeInTheDocument()
  })

  it('M4: autosaves title edits and commits the draft page order', async () => {
    renderEditor()
    await screen.findByTestId('deck-pages')
    window.dispatchEvent(new CustomEvent('media-editor-title-change', { detail: 'Updated Sunday' }))
    await waitFor(() => {
      expect(commitDeck).toHaveBeenCalledWith(expect.anything(), 'media:deck', {
        revision_id: 'rev1',
        page_ids: ['p1', 'p2'],
        section_titles: ['Section 1', null],
      })
    }, { timeout: 2_000 })
  })

  it('M5: does not autosave an empty deck', async () => {
    fetchMedia.mockResolvedValue(deckMedia({
      pending_revision: { revision_id: 'rev1', pages: [] },
    }))
    renderEditor()
    await screen.findByTestId('deck-pages')
    await new Promise((resolve) => setTimeout(resolve, 800))
    expect(commitDeck).not.toHaveBeenCalled()
  })

  it('commits uploaded pages at the selected insertion boundary', async () => {
    const user = userEvent.setup()
    uploadMediaSource.mockResolvedValue(
      deckMedia({
        pending_revision: {
          revision_id: 'rev1',
          pages: [
            { id: 'p1', blob_id: 'b1', section_title: 'Section 1' },
            { id: 'p2', blob_id: 'b2' },
            { id: 'p-new', blob_id: 'b-new' },
          ],
        },
      }),
    )
    renderEditor()
    await screen.findByTestId('deck-pages')

    await user.click(screen.getByRole('button', { name: 'add at boundary 1' }))

    await waitFor(() => {
      expect(commitDeck).toHaveBeenCalledWith(expect.anything(), 'media:deck', {
        revision_id: 'rev1',
        page_ids: ['p1', 'p-new', 'p2'],
        section_titles: ['Section 1', null, null],
      })
    })
  })

})
