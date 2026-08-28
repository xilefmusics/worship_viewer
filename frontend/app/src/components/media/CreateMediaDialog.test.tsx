import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { CreateMediaDialog } from '@/components/media/CreateMediaDialog'
import type { CreateMediaKind } from '@/lib/media-display'

const mocks = vi.hoisted(() => ({
  createMedia: vi.fn(),
  createUploadedMedia: vi.fn(),
}))

vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}))
vi.mock('@/hooks/useWritableTeams', () => ({
  useWritableTeams: () => ({
    teams: [{ id: 'team:1', name: 'Team', members: [] }],
    user: { id: 'user:1' },
    isPending: false,
  }),
}))
vi.mock('@/api/media', () => ({
  createMedia: mocks.createMedia,
  mediaListRootKey: ['media'],
}))
vi.mock('@/api/media-upload', () => ({
  createUploadedMedia: mocks.createUploadedMedia,
}))
vi.mock('@/components/media/MediaFields', () => ({
  MediaFields: ({
    title,
    onTitleChange,
    onKindChange,
    onFilesChange,
  }: {
    title: string
    onTitleChange: (value: string) => void
    onKindChange: (value: CreateMediaKind) => void
    onFilesChange: (value: File[]) => void
  }) => (
    <div>
      <label htmlFor="media-title">media.fields.title</label>
      <input id="media-title" value={title} onChange={(event) => onTitleChange(event.target.value)} />
      <button type="button" onClick={() => onKindChange('slide_deck')}>set-slide-deck</button>
      <button type="button" onClick={() => onFilesChange([new File(['%PDF-1.5'], 'slides.pdf', { type: 'application/pdf' })])}>choose-deck-file</button>
    </div>
  ),
}))

function renderDialog(onCreated = vi.fn()) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={client}>
      <CreateMediaDialog open onOpenChange={vi.fn()} onCreated={onCreated} />
    </QueryClientProvider>,
  )
}

describe('CreateMediaDialog', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mocks.createUploadedMedia.mockResolvedValue({
      id: 'media:deck',
      owner: 'team:1',
      title: 'Sunday slides',
      content: { type: 'slide_deck', pages: [{ blob_id: 'blob:1' }] },
    })
  })

  it('M1: requires files before creating a slide deck', async () => {
    const user = userEvent.setup()
    renderDialog()
    await user.click(screen.getByRole('button', { name: 'set-slide-deck' }))
    await user.type(screen.getByLabelText('media.fields.title'), 'Sunday slides')
    await user.click(screen.getByRole('button', { name: 'media.actions.create' }))
    expect(await screen.findByRole('alert')).toHaveTextContent('media.validation.fileRequired')
  })

  it('creates an uploaded slide deck atomically before reporting it as created', async () => {
    const user = userEvent.setup()
    const onCreated = vi.fn()
    renderDialog(onCreated)
    await user.click(screen.getByRole('button', { name: 'set-slide-deck' }))
    await user.click(screen.getByRole('button', { name: 'choose-deck-file' }))
    await user.type(screen.getByLabelText('media.fields.title'), 'Sunday slides')
    await user.click(screen.getByRole('button', { name: 'media.actions.create' }))

    expect(await screen.findByText('media.upload.progress')).toBeInTheDocument()
    expect(mocks.createUploadedMedia).toHaveBeenCalledWith(expect.objectContaining({
      kind: 'slide_deck',
      title: 'Sunday slides',
      owner: 'team:1',
    }))
    expect(onCreated).toHaveBeenCalledWith(
      'media:deck',
      expect.objectContaining({ content: expect.objectContaining({ type: 'slide_deck' }) }),
    )
  })
})
