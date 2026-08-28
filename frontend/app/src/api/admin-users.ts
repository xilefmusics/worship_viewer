import type { QueryClient } from '@tanstack/react-query'

import { api } from '@/api/client'
import { problemMessageFromBody } from '@/api/problem'
import type { components } from '@/api/schema'
import { redirectToLoginAfterUnauthorized } from '@/lib/api-unauthorized'
import { parseTotalCount } from '@/lib/list-pagination'

export type AdminUser = components['schemas']['User']
export const ADMIN_USERS_PAGE_SIZE = 50

export async function fetchAdminUsersPage(
  queryClient: QueryClient,
  args: { page: number; q: string; signal?: AbortSignal },
): Promise<{ items: AdminUser[]; total: number | undefined }> {
  const result = await api.GET('/api/v1/users', {
    params: {
      query: {
        page: args.page,
        page_size: ADMIN_USERS_PAGE_SIZE,
        q: args.q.trim() || undefined,
      },
    },
    signal: args.signal,
  })

  if (result.response.status === 401) {
    await redirectToLoginAfterUnauthorized(queryClient)
  }
  if (!result.response.ok || result.data == null) {
    throw new Error(problemMessageFromBody(result.error, 'Could not load users.'))
  }

  return { items: result.data, total: parseTotalCount(result.response) }
}
