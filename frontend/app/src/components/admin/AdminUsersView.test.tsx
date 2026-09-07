import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest'

import { AdminUsersView } from '@/components/admin/AdminUsersView'
import { renderWithProviders } from '@/test/renderWithProviders'

const mocks = vi.hoisted(() => ({
  fetchPage: vi.fn(),
  fetchImpersonationStatus: vi.fn(),
  startImpersonation: vi.fn(),
  clearAllLocalData: vi.fn(),
  q: '',
  setQInput: vi.fn(),
}))

vi.mock('@/api/admin-users', async (importOriginal) => {
  const original = await importOriginal<typeof import('@/api/admin-users')>()
  return { ...original, fetchAdminUsersPage: mocks.fetchPage }
})
vi.mock('@/api/impersonation', async (importOriginal) => {
  const original = await importOriginal<typeof import('@/api/impersonation')>()
  return {
    ...original,
    fetchImpersonationStatus: mocks.fetchImpersonationStatus,
    startImpersonation: mocks.startImpersonation,
  }
})
vi.mock('@/lib/clear-local', () => ({ clearAllLocalData: mocks.clearAllLocalData }))
vi.mock('@/hooks/useHubSearch', () => ({
  useHubSearch: () => ({ debouncedQ: mocks.q, setQInput: mocks.setQInput }),
}))
vi.mock('@/context/HubScrollContainerContext', () => ({
  useHubScrollContainerRef: () => ({ current: null }),
}))

const admin = {
  id: 'user:admin',
  email: 'admin@example.com',
  role: 'admin' as const,
  created_at: '2026-08-28T10:00:00Z',
}

beforeEach(() => {
  mocks.q = ''
  mocks.setQInput.mockReset()
  mocks.fetchPage.mockReset().mockResolvedValue({ items: [admin], total: 51 })
  mocks.fetchImpersonationStatus.mockReset().mockResolvedValue({ enabled: true, active: false })
  mocks.startImpersonation.mockReset().mockResolvedValue({ enabled: true, active: true })
  mocks.clearAllLocalData.mockReset().mockResolvedValue(undefined)
})

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('AdminUsersView', () => {
  it('renders hub-style rows and loads the next page', async () => {
    const user = userEvent.setup()
    const view = renderWithProviders(<AdminUsersView />)

    expect(await screen.findByText('admin@example.com')).toBeInTheDocument()
    expect(screen.getByText(/Admin/)).toBeInTheDocument()
    expect(screen.queryByText('user:admin')).not.toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Load more' })).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'Load more' }))
    await waitFor(() =>
      expect(mocks.fetchPage).toHaveBeenLastCalledWith(
        expect.anything(),
        expect.objectContaining({ page: 1, q: '' }),
      ),
    )

    mocks.q = 'singer'
    view.rerender(<AdminUsersView />)
    await waitFor(() =>
      expect(mocks.fetchPage).toHaveBeenLastCalledWith(
        expect.anything(),
        expect.objectContaining({ page: 0, q: 'singer' }),
      ),
    )
  })

  it('shows search-specific empty state and clears the shared search', async () => {
    const user = userEvent.setup()
    mocks.q = 'missing'
    mocks.fetchPage.mockResolvedValue({ items: [], total: 0 })
    renderWithProviders(<AdminUsersView />)

    expect(await screen.findByText('No matching users')).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Clear search' }))
    expect(mocks.setQInput).toHaveBeenCalledWith('')
  })

  it('offers retry after a failed request', async () => {
    const user = userEvent.setup()
    mocks.fetchPage.mockRejectedValueOnce(new Error('Directory unavailable.'))
    renderWithProviders(<AdminUsersView />)

    expect(await screen.findByText('Directory unavailable.')).toBeInTheDocument()
    mocks.fetchPage.mockResolvedValue({ items: [admin], total: 1 })
    await user.click(screen.getByRole('button', { name: 'Retry' }))
    expect(await screen.findByText('admin@example.com')).toBeInTheDocument()
  })

  it('starts impersonation from the actions drawer without a confirmation dialog', async () => {
    const assign = vi.fn()
    vi.stubGlobal('location', { ...window.location, assign })
    const user = userEvent.setup()
    renderWithProviders(<AdminUsersView />)

    await user.click(await screen.findByRole('button', { name: 'Actions for admin@example.com' }))
    expect(screen.getByRole('dialog', { name: 'admin@example.com' })).toBeInTheDocument()
    await user.click(await screen.findByRole('menuitem', { name: 'Impersonate' }))
    expect(screen.queryByText('Impersonate this user?')).not.toBeInTheDocument()
    await waitFor(() => expect(mocks.startImpersonation).toHaveBeenCalledWith('user:admin'))
    await waitFor(() => expect(mocks.clearAllLocalData).toHaveBeenCalled())
    await waitFor(() => expect(assign).toHaveBeenCalledWith('/collections'))
  })
})
