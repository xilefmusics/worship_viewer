import * as Dialog from '@radix-ui/react-dialog'
import { useInfiniteQuery, useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { fetchMediaPage, mediaListKey, type Media } from '@/api/media'
import { fetchTeamsPage, type Team } from '@/api/teams-sessions-fetch'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { useDebounced } from '@/hooks/useSongPickerQuery'
import { useSession } from '@/hooks/useSession'
import { getNextPageIndex } from '@/lib/list-pagination'
import { mediaDisplayKind } from '@/lib/media-display'
import { getTeamDisplayName } from '@/lib/team-display-name'
import { teamsListRootKey } from '@/lib/teams-sessions-keys'

const ALL_TEAMS = '__all__'

export function SetlistMediaPickerSheet({
  open,
  onOpenChange,
  blockedAdd = false,
  duplicateCountFor,
  onPickMedia,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  blockedAdd?: boolean
  duplicateCountFor: (mediaId: string) => number
  onPickMedia: (media: Media) => void
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [q, setQ] = useState('')
  const [teamId, setTeamId] = useState<string>(ALL_TEAMS)
  const debouncedQ = useDebounced(300, q)
  const { data: user } = useSession()
  const teamsQuery = useInfiniteQuery({
    queryKey: [...teamsListRootKey, 'setlistMediaPicker'],
    enabled: open,
    initialPageParam: 0,
    queryFn: ({ pageParam, signal }) => fetchTeamsPage(queryClient, { page: pageParam as number, q: '', signal }),
    getNextPageParam: (_last, all) => getNextPageIndex(all),
  })
  const teams = useMemo(() => teamsQuery.data?.pages.flatMap((page) => page.items) as Team[] | undefined ?? [], [teamsQuery.data?.pages])
  const selectedTeam = teamId === ALL_TEAMS ? null : teamId
  const query = useInfiniteQuery({
    queryKey: [...mediaListKey(debouncedQ, selectedTeam), 'picker'],
    enabled: open,
    initialPageParam: 0,
    queryFn: ({ pageParam, signal }) =>
      fetchMediaPage(queryClient, {
        page: pageParam as number,
        q: debouncedQ,
        teamId: selectedTeam,
        signal,
      }),
    getNextPageParam: (_last, all) => getNextPageIndex(all),
  })
  const items = useMemo(
    () => query.data?.pages.flatMap((page) => page.items).filter((media) => media.status === 'ready' && media.content != null) ?? [],
    [query.data?.pages],
  )

  const close = () => {
    setQ('')
    onOpenChange(false)
  }

  return (
    <Dialog.Root open={open} onOpenChange={(next) => (next ? onOpenChange(true) : close())}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-[60] bg-black/40" />
        <Dialog.Content className="fixed inset-x-0 bottom-0 z-[61] mx-auto flex max-h-[min(36rem,88dvh)] w-full max-w-2xl flex-col gap-3 rounded-t-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 text-[var(--color-foreground)] shadow-[var(--shadow-elevated)]">
          <div className="mx-auto h-1.5 w-12 shrink-0 rounded-full bg-[var(--color-muted)]" aria-hidden />
          <div className="flex items-center justify-between gap-2">
            <Dialog.Title className="text-base font-semibold">{t('setlists.editor.addMediaTitle')}</Dialog.Title>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={() => {
                close()
                void navigate({ to: '/media' })
              }}
            >
              {t('setlists.editor.manageMedia')}
            </Button>
          </div>
          <div className="grid gap-2 sm:grid-cols-[minmax(0,1fr)_12rem]">
            <Input
              value={q}
              onChange={(event) => setQ(event.target.value)}
              placeholder={t('setlists.editor.mediaPickerSearchPlaceholder')}
              aria-label={t('setlists.editor.mediaPickerSearchAria')}
              autoComplete="off"
            />
            <Select value={teamId} onValueChange={setTeamId}>
              <SelectTrigger aria-label={t('setlists.editor.mediaPickerTeamAria')}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={ALL_TEAMS}>{t('setlists.editor.mediaPickerAllTeams')}</SelectItem>
                {teams.map((team) => (
                  <SelectItem key={team.id} value={team.id}>
                    {getTeamDisplayName(team, user?.id, t)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="min-h-0 flex-1 overflow-y-auto">
            {query.isError ? <p role="alert" className="py-5 text-center text-sm text-[var(--color-danger)]">{t('setlists.editor.mediaPickerError')}</p> : null}
            {query.isPending ? <p role="status" className="py-5 text-center text-sm text-[var(--color-muted-foreground)]">{t('common.load')}</p> : null}
            {!query.isPending && !query.isError && items.length === 0 ? <p className="py-5 text-center text-sm text-[var(--color-muted-foreground)]">{t('setlists.editor.mediaPickerEmpty')}</p> : null}
            <ul className="grid gap-1 pb-4">
              {items.map((media) => {
                const duplicateCount = duplicateCountFor(media.id)
                const kind = mediaDisplayKind(media)
                return (
                  <li key={media.id}>
                    <button
                      type="button"
                      disabled={blockedAdd}
                      title={blockedAdd ? t('setlists.editor.waitForSave') : undefined}
                      className="flex w-full items-center justify-between gap-3 rounded-lg px-3 py-2 text-left hover:bg-[var(--color-muted)] disabled:cursor-not-allowed disabled:opacity-55"
                      onClick={() => {
                        if (blockedAdd) return
                        onPickMedia(media)
                        close()
                      }}
                    >
                      <span className="min-w-0">
                        <span className="block truncate text-sm font-medium">{media.title}</span>
                        <span className="block text-xs text-[var(--color-muted-foreground)]">{t(`media.kinds.${kind}`)} · {t('media.states.ready')}</span>
                      </span>
                      {duplicateCount > 0 ? <span className="shrink-0 text-[0.65rem] uppercase text-[var(--color-muted-foreground)]">{t('common.duplicateBadge', { container: t('common.containerSetlist'), count: duplicateCount })}</span> : null}
                    </button>
                  </li>
                )
              })}
            </ul>
            {query.hasNextPage ? <div className="flex justify-center pb-3"><Button type="button" size="sm" variant="outline" disabled={query.isFetchingNextPage} onClick={() => void query.fetchNextPage()}>{query.isFetchingNextPage ? t('common.load') : t('hub.loadMore')}</Button></div> : null}
          </div>
          <Dialog.Close asChild><Button type="button" variant="outline">{t('common.cancel')}</Button></Dialog.Close>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
