import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'

import { DeckPagesEditor } from '@/components/media/DeckPagesEditor'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, options?: Record<string, number>) =>
      options?.number != null ? `${key}:${options.number}` : key,
  }),
}))
vi.mock('@/components/media/DeckPagePreview', () => ({
  DeckPagePreview: ({ label }: { label: string }) => <div>{label}</div>,
}))

describe('DeckPagesEditor', () => {
  it('M3: lists pages with reorder handles and can remove a page', async () => {
    const user = userEvent.setup()
    const onReorder = vi.fn()
    const onRemove = vi.fn()
    render(
      <DeckPagesEditor
        mediaId="media:1"
        pages={[
          { id: 'p1', blob_id: 'b1' },
          { id: 'p2', blob_id: 'b2' },
        ]}
        onReorder={onReorder}
        onRemove={onRemove}
        onReplace={vi.fn()}
        onAdd={vi.fn()}
      />,
    )
    expect(screen.getByRole('list', { name: 'media.deck.listAria' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'media.deck.reorderHandle:1' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'media.deck.reorderHandle:2' })).toBeInTheDocument()
    await user.click(screen.getAllByRole('button', { name: 'media.deck.removePage' })[0])
    expect(onRemove).toHaveBeenCalledWith('p1')
  })

  it('M5: shows an empty-guard hint when there are no pages', () => {
    render(
      <DeckPagesEditor
        mediaId="media:1"
        pages={[]}
        onReorder={vi.fn()}
        onRemove={vi.fn()}
        onReplace={vi.fn()}
        onAdd={vi.fn()}
      />,
    )
    expect(screen.getByText('media.deck.empty')).toBeInTheDocument()
  })
})
