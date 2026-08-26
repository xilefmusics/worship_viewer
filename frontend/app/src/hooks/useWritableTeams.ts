import { useInfiniteQuery, useQueryClient } from '@tanstack/react-query'
import { useMemo } from 'react'

import { fetchTeamsPage, type Team } from '@/api/teams-sessions-fetch'
import { useSession } from '@/hooks/useSession'
import { getNextPageIndex } from '@/lib/list-pagination'
import { canEditTeamLibrary } from '@/lib/team-permissions'
import { teamsListRootKey } from '@/lib/teams-sessions-keys'

export function useWritableTeams(scope: string, enabled = true) {
  const queryClient = useQueryClient()
  const { data: user } = useSession()
  const query = useInfiniteQuery({
    queryKey: [...teamsListRootKey, 'writable', scope] as const,
    initialPageParam: 0,
    enabled: enabled && Boolean(user?.id),
    queryFn: ({ pageParam, signal }) =>
      fetchTeamsPage(queryClient, { page: pageParam as number, q: '', signal }),
    getNextPageParam: (_last, all) => getNextPageIndex(all),
  })
  const teams = useMemo(() => {
    if (!user?.id) return []
    return (query.data?.pages.flatMap((page) => page.items) ?? []).filter((team: Team) =>
      canEditTeamLibrary(team, user.id),
    )
  }, [query.data?.pages, user])
  return { ...query, teams, user }
}
