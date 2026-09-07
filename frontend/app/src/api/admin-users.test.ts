import { beforeEach, describe, expect, it, vi } from 'vitest'

import { api } from '@/api/client'
import { fetchAdminUsersPage, getAdminUsersNextPageParam } from '@/api/admin-users'

vi.mock('@/api/client', () => ({ api: { GET: vi.fn() } }))
vi.mock('@/lib/api-unauthorized', () => ({ redirectToLoginAfterUnauthorized: vi.fn() }))

const queryClient = {} as never
const response = (status = 200, headers?: Record<string, string>) =>
  new Response(null, { status, headers })
const user = {
  id: 'user:1',
  email: 'admin@example.com',
  role: 'admin' as const,
  created_at: '2026-08-28T10:00:00Z',
}

beforeEach(() => vi.clearAllMocks())

describe('Admin users API', () => {
  it('maps trimmed search, pagination, and total count', async () => {
    vi.mocked(api.GET).mockResolvedValue({
      data: [user],
      response: response(200, { 'X-Total-Count': '73' }),
    } as never)

    await expect(
      fetchAdminUsersPage(queryClient, { page: 1, q: ' admin@example.com ' }),
    ).resolves.toEqual({ items: [user], total: 73 })
    expect(api.GET).toHaveBeenCalledWith(
      '/api/v1/users',
      expect.objectContaining({
        params: { query: { page: 1, page_size: 50, q: 'admin@example.com' } },
      }),
    )
  })

  it('omits a whitespace-only search and surfaces problem details', async () => {
    vi.mocked(api.GET)
      .mockResolvedValueOnce({ data: [], response: response() } as never)
      .mockResolvedValueOnce({
        error: { title: 'User list failed', detail: 'Directory unavailable.' },
        response: response(500),
      } as never)

    await fetchAdminUsersPage(queryClient, { page: 0, q: '   ' })
    expect(api.GET).toHaveBeenLastCalledWith(
      '/api/v1/users',
      expect.objectContaining({
        params: { query: { page: 0, page_size: 50, q: undefined } },
      }),
    )
    await expect(fetchAdminUsersPage(queryClient, { page: 0, q: '' })).rejects.toThrow(
      'Directory unavailable.',
    )
  })
})

describe('getAdminUsersNextPageParam', () => {
  it('uses the total header when present', () => {
    expect(getAdminUsersNextPageParam({ items: [user], total: 51 }, [{ items: [user], total: 51 }])).toBe(1)
    expect(getAdminUsersNextPageParam({ items: [user], total: 1 }, [{ items: [user], total: 1 }])).toBeUndefined()
  })

  it('falls back to a full page when total is missing', () => {
    const fullPage = { items: Array.from({ length: 50 }, (_, i) => i), total: undefined }
    expect(getAdminUsersNextPageParam(fullPage, [fullPage])).toBe(1)
    expect(
      getAdminUsersNextPageParam({ items: [user], total: undefined }, [{ items: [user], total: undefined }]),
    ).toBeUndefined()
  })
})
