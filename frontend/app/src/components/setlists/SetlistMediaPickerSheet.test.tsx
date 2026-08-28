import { fireEvent, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import type { Media } from '@/api/media'
import { SetlistMediaPickerSheet } from '@/components/setlists/SetlistMediaPickerSheet'
import { renderWithProviders } from '@/test/renderWithProviders'

const mocks = vi.hoisted(() => ({
  fetchMediaPage: vi.fn(),
  fetchTeamsPage: vi.fn(),
  createUploadedMedia: vi.fn(),
  navigate: vi.fn(),
}))

vi.mock('@tanstack/react-router', () => ({ useNavigate: () => mocks.navigate }))
vi.mock('@/api/media', async (importOriginal) => {
  const original = await importOriginal<typeof import('@/api/media')>()
  return { ...original, fetchMediaPage: mocks.fetchMediaPage }
})
vi.mock('@/api/media-upload', () => ({ createUploadedMedia: mocks.createUploadedMedia }))
vi.mock('@/api/teams-sessions-fetch', async (importOriginal) => {
  const original = await importOriginal<typeof import('@/api/teams-sessions-fetch')>()
  return { ...original, fetchTeamsPage: mocks.fetchTeamsPage }
})
vi.mock('@/hooks/useSession', () => ({ useSession: () => ({ data: { id: 'user:1' } }) }))
vi.mock('@/components/media/CreateMediaDialog', () => ({
  CreateMediaDialog: ({ open, onCreated }: { open: boolean; onCreated: (id: string, media: Media) => void }) => open ? (
    <div role="dialog" aria-label="create media dialog">
      <button type="button" onClick={() => onCreated('media:created', media('media:created'))}>finish create</button>
    </div>
  ) : null,
}))

function media(id: string): Media {
  return { id, title: `Title ${id}`, owner: 'team:1', content: { type: 'web_page', url: 'https://example.com' } }
}

function renderPicker(options: { blockedAdd?: boolean; onPickMedia?: (item: Media) => void } = {}) {
  return renderWithProviders(
    <SetlistMediaPickerSheet
      open
      onOpenChange={vi.fn()}
      blockedAdd={options.blockedAdd}
      defaultOwner="team:1"
      duplicateCountFor={() => 0}
      onPickMedia={options.onPickMedia ?? vi.fn()}
    />,
  )
}

describe('SetlistMediaPickerSheet create flow', () => {
  beforeEach(() => {
    mocks.fetchMediaPage.mockResolvedValue({ items: [], total: 0 })
    mocks.fetchTeamsPage.mockResolvedValue({ items: [], total: 0 })
    mocks.createUploadedMedia.mockResolvedValue(media('media:dropped'))
  })

  it('offers creation when adding is allowed and disables it while adding is blocked', () => {
    const { unmount } = renderPicker()
    expect(screen.getByRole('button', { name: 'Create media' })).toBeEnabled()
    unmount()
    renderPicker({ blockedAdd: true })
    expect(screen.getByRole('button', { name: 'Create media' })).toBeDisabled()
  })

  it('shows complete stored media and a quick-upload row', async () => {
    mocks.fetchMediaPage.mockResolvedValue({ items: [media('media:library')], total: 1 })
    renderPicker()
    await screen.findByText('Title media:library')
    const rows = within(screen.getByRole('list')).getAllByRole('listitem')
    expect(rows.at(-1)).toHaveTextContent('Drop here')
  })

  it('creates and appends an uploaded media item in one synchronous request', async () => {
    const onPickMedia = vi.fn()
    renderPicker({ onPickMedia })
    const file = new File(['video'], 'welcome.mp4', { type: 'video/mp4' })
    fireEvent.drop(screen.getByText('Drop here').closest('label')!, { dataTransfer: { files: [file] } })
    await waitFor(() => expect(mocks.createUploadedMedia).toHaveBeenCalledWith(expect.objectContaining({
      kind: 'video', title: 'welcome', owner: 'team:1', files: [file],
    })))
    await waitFor(() => expect(onPickMedia).toHaveBeenCalledWith(expect.objectContaining({ id: 'media:dropped' })))
  })

  it('appends a synchronously created URL item without navigating', async () => {
    const user = userEvent.setup()
    const onPickMedia = vi.fn()
    renderPicker({ onPickMedia })
    await user.click(screen.getByRole('button', { name: 'Create media' }))
    fireEvent.click(screen.getByRole('button', { name: 'finish create' }))
    await waitFor(() => expect(onPickMedia).toHaveBeenCalledWith(expect.objectContaining({ id: 'media:created' })))
    expect(mocks.navigate).not.toHaveBeenCalled()
  })
})
