import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'

import { CreateMediaDialog } from '@/components/media/CreateMediaDialog'
import type { CreateMediaKind } from '@/lib/media-display'

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
  createMedia: vi.fn(),
  mediaListRootKey: ['media'],
}))
vi.mock('@/components/media/MediaFields', () => ({
  MediaFields: ({
    title,
    onTitleChange,
    onKindChange,
  }: {
    title: string
    onTitleChange: (value: string) => void
    onKindChange: (value: CreateMediaKind) => void
  }) => (
    <div>
      <label htmlFor="media-title">media.fields.title</label>
      <input id="media-title" value={title} onChange={(event) => onTitleChange(event.target.value)} />
      <button type="button" onClick={() => onKindChange('slide_deck')}>set-slide-deck</button>
    </div>
  ),
}))

function renderDialog() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={client}>
      <CreateMediaDialog open onOpenChange={vi.fn()} onCreated={vi.fn()} />
    </QueryClientProvider>,
  )
}

describe('CreateMediaDialog', () => {
  it('M1: requires files before creating a slide deck', async () => {
    const user = userEvent.setup()
    renderDialog()
    await user.click(screen.getByRole('button', { name: 'set-slide-deck' }))
    await user.type(screen.getByLabelText('media.fields.title'), 'Sunday slides')
    await user.click(screen.getByRole('button', { name: 'media.actions.create' }))
    expect(await screen.findByRole('alert')).toHaveTextContent('media.validation.fileRequired')
  })
})
