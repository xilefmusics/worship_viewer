import { useQueryClient } from '@tanstack/react-query'
import { useNavigate, useRouterState } from '@tanstack/react-router'
import * as Dialog from '@radix-ui/react-dialog'
import { AnimatePresence, motion, useReducedMotion } from 'motion/react'
import {
  memo,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent,
  type ReactNode,
} from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

import { CopyIcon } from '@/components/icons/lucide-animated/copy-icon'
import { DownloadIcon } from '@/components/icons/lucide-animated/download-icon'
import { EllipsisIcon } from '@/components/icons/lucide-animated/ellipsis-icon'
import { FileStackIcon } from '@/components/icons/lucide-animated/file-stack-icon'
import { FileTextIcon } from '@/components/icons/lucide-animated/file-text-icon'
import { FolderXIcon } from '@/components/icons/lucide-animated/folder-x-icon'
import { ListMusicIcon } from '@/components/icons/lucide-animated/list-music-icon'
import { OutputIcon } from '@/components/icons/lucide-animated/output-icon'
import { PencilIcon } from '@/components/icons/lucide-animated/pencil-icon'
import { PrinterIcon } from '@/components/icons/lucide-animated/printer-icon'
import { ProjectorIcon } from '@/components/icons/lucide-animated/projector-icon'
import { TrashIcon } from '@/components/icons/lucide-animated/trash-icon'
import { XIcon } from '@/components/icons/lucide-animated/x-icon'
import { AddSongToSetlistDialog } from '@/components/hub/AddSongToSetlistDialog'
import { SetlistItemCounts } from '@/components/hub/SetlistItemCounts'
import {
  HUB_LIST_AVATAR_CLASS,
  HUB_LIST_ROW_BORDER_CLASS,
  HUB_LIST_ROW_INSET_LAST_CLASS,
  HUB_LIST_ROW_SHELL_CLASS,
  HUB_LIST_ROW_TEXT_COLUMN_CLASS,
  HUB_LIST_SUBTITLE_CLASS,
  HUB_LIST_TITLE_CLASS,
} from '@/components/hub/hub-list-styles'

import type { Collection, Setlist, Song } from '@/api/list-fetch'
import { useHubScrollContainerRef } from '@/context/HubScrollContainerContext'
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { useChordFormatPreference } from '@/hooks/useChordFormatPreference'
import { useHideChordsPreference } from '@/hooks/useHideChordsPreference'
import { useHubSearch } from '@/hooks/useHubSearch'
import { useCoverImageSrc } from '@/hooks/useCoverImageSrc'
import { useDeleteHubEntity, HubDeleteConflictError } from '@/hooks/useDeleteHubEntity'
import { useInfiniteHubList } from '@/hooks/useInfiniteHubList'
import { downloadPlayerForOffline, removeOfflinePlayerCopy } from '@/lib/offline/download-player-offline'
import { useOnline } from '@/hooks/use-online'
import { useSession } from '@/hooks/useSession'
import { useTeamDetail } from '@/hooks/useTeamDetail'
import { observeElementIntersection } from '@/lib/browser-apis'
import { exportPdfHintTitle } from '@/lib/export-pdf-hint'
import { runCollectionExport } from '@/lib/run-collection-export'
import { runSetlistExport } from '@/lib/run-setlist-export'
import { runSongExport, type SongExportKind } from '@/lib/run-song-export'
import { duplicateCollection, duplicateSetlist } from '@/lib/duplicate-hub-entity'
import type { HubEntity } from '@/lib/hub-entity'
import { hubEntityEditSplat } from '@/lib/hub-entity-edit'
import { hubListKey, hubListRootKey } from '@/lib/hub-list-keys'
import { useHubListsUpdatedAt } from '@/hooks/useHubListsUpdatedAt'
import { hubEntityToPlayerType, buildPlayerSearch } from '@/lib/player-route'
import { readPlayerDefaultMode } from '@/lib/player/player-mode-preference'
import { emptyEditorReturnSearch } from '@/lib/player/player-editor-return'
import { useHubViewMode } from '@/hooks/useHubViewMode'
import { useMediaQuery } from '@/hooks/useMediaQuery'
import { resolveCollectionsLayoutMode } from '@/lib/hub-view-mode'
import { getTeamDisplayName } from '@/lib/team-display-name'
import { cn } from '@/lib/utils'

/** Card grid: dense on laptop+ (6 → 8 cols), stays 2 cols on narrow phones. */
const hubCardGridClass =
  'grid grid-cols-2 gap-2 pb-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 xl:grid-cols-8'

type EntityListViewProps = {
  entity: HubEntity
}

function songTitle(song: Song): string {
  const t = song.data.titles?.[0]
  return t?.trim() ? t : '—'
}

function songSubtitle(song: Song, unknownArtist: string): string {
  const a = (song.data.artists ?? []).filter(Boolean).join(', ')
  return a || unknownArtist
}

const tapFeedback = { scale: 0.985 }
const tapTransition = { duration: 0.12, ease: [0.25, 0.1, 0.25, 1] as const }

export function EntityListView({ entity }: EntityListViewProps) {
  const { t } = useTranslation()
  const { debouncedQ, selectedTeamId, setQInput } = useHubSearch()
  const reduceMotion = useReducedMotion()
  const queryClient = useQueryClient()
  const scrollRef = useHubScrollContainerRef()
  const sentinelRef = useRef<HTMLDivElement>(null)
  const [pullVisual, setPullVisual] = useState(0)
  const [ptrRefreshing, setPtrRefreshing] = useState(false)
  const pullStartRef = useRef<number | null>(null)
  const pullDyRef = useRef(0)

  const { viewMode: collectionsViewPreference } = useHubViewMode('collections')
  const isLandscape = useMediaQuery('(orientation: landscape)')
  const viewMode =
    entity === 'collections'
      ? resolveCollectionsLayoutMode(collectionsViewPreference, isLandscape)
      : 'list'
  const pathname = useRouterState({ select: (s) => s.location.pathname })

  useEffect(() => {
    if (entity !== 'setlists' && entity !== 'collections') return
    if (entity === 'setlists' && pathname !== '/setlists') return
    if (entity === 'collections' && pathname !== '/collections') return
    scrollRef.current?.scrollTo({ top: 0, behavior: 'instant' })
  }, [entity, pathname, scrollRef])

  const {
    data,
    error,
    isPending,
    isFetchingNextPage,
    hasNextPage,
    fetchNextPage,
    refetch,
  } = useInfiniteHubList(entity)

  const deleteMutation = useDeleteHubEntity(entity)
  const networkOnline = useOnline()
  const [deleteTarget, setDeleteTarget] = useState<{
    id: string
    label: string
    songCount?: number
  } | null>(null)
  const listsUpdatedAt = useHubListsUpdatedAt(!networkOnline, entity, Boolean(data))

  const deleteBlocked =
    entity === 'collections' && deleteTarget != null && (deleteTarget.songCount ?? 0) > 0

  const items = useMemo(() => {
    const pages = (data?.pages ?? []) as Array<{
      items: (Collection | Song | Setlist)[]
      total: number | undefined
    }>
    const flat = pages.flatMap((p) => p.items)
    if (entity !== 'setlists') return flat
    return [...(flat as Setlist[])].sort((a, b) =>
      b.title.localeCompare(a.title, undefined, { numeric: true }),
    )
  }, [data?.pages, entity])

  const runPullRefresh = useCallback(async () => {
    if (!networkOnline) {
      toast.info(t('hub.refresh.offlineBlocked'))
      return
    }
    await queryClient.resetQueries({ queryKey: hubListKey(entity, debouncedQ, selectedTeamId) })
    await refetch()
    scrollRef.current?.scrollTo({ top: 0, behavior: 'smooth' })
  }, [queryClient, entity, debouncedQ, selectedTeamId, refetch, scrollRef, networkOnline, t])

  useEffect(() => {
    const el = scrollRef.current
    if (!el) return
    // Non-passive touchmove on the scrollport breaks wheel / trackpad scrolling in desktop Chromium.
    if (typeof navigator !== 'undefined' && navigator.maxTouchPoints === 0) return

    let touchMoveListener: ((e: TouchEvent) => void) | null = null

    const onTouchStart = (e: TouchEvent) => {
      if (el.scrollTop > 0) return
      if (el.scrollHeight <= el.clientHeight) return
      pullStartRef.current = e.touches[0].clientY

      if (touchMoveListener) return
      touchMoveListener = (moveEvent: TouchEvent) => {
        if (pullStartRef.current == null) return
        if (el.scrollHeight <= el.clientHeight) {
          pullStartRef.current = null
          pullDyRef.current = 0
          setPullVisual(0)
          return
        }
        if (el.scrollTop > 0) {
          pullStartRef.current = null
          pullDyRef.current = 0
          setPullVisual(0)
          return
        }
        const dy = moveEvent.touches[0].clientY - pullStartRef.current
        if (dy > 0) {
          moveEvent.preventDefault()
          pullDyRef.current = Math.min(dy, 72)
          setPullVisual(pullDyRef.current)
        }
      }
      el.addEventListener('touchmove', touchMoveListener, { passive: false })
    }

    const onTouchEnd = () => {
      if (touchMoveListener) {
        el.removeEventListener('touchmove', touchMoveListener)
        touchMoveListener = null
      }
      if (pullStartRef.current == null) return
      pullStartRef.current = null
      const d = pullDyRef.current
      pullDyRef.current = 0
      setPullVisual(0)
      if (d <= 40) return
      setPtrRefreshing(true)
      void runPullRefresh().finally(() => setPtrRefreshing(false))
    }

    el.addEventListener('touchstart', onTouchStart, { passive: true })
    el.addEventListener('touchend', onTouchEnd)
    el.addEventListener('touchcancel', onTouchEnd)

    return () => {
      if (touchMoveListener) {
        el.removeEventListener('touchmove', touchMoveListener)
      }
      el.removeEventListener('touchstart', onTouchStart)
      el.removeEventListener('touchend', onTouchEnd)
      el.removeEventListener('touchcancel', onTouchEnd)
    }
  }, [runPullRefresh, scrollRef])

  useEffect(() => {
    const root = scrollRef.current
    const sentinel = sentinelRef.current
    if (!root || !sentinel) return

    return observeElementIntersection(
      sentinel,
      (entries) => {
        const hit = entries[0]?.isIntersecting
        if (hit && hasNextPage && !isFetchingNextPage) {
          void fetchNextPage()
        }
      },
      { root, rootMargin: '120px' },
    )
  }, [hasNextPage, isFetchingNextPage, fetchNextPage, items.length, scrollRef])

  const showSkeleton = isPending && !data

  return (
    <>
      <div className="relative flex w-full min-w-0 flex-col">
        {!networkOnline && listsUpdatedAt ? (
          <p className="mb-2 text-center text-xs text-[var(--color-muted-foreground)]">
            {t('hub.offline.lastUpdated', {
              when: new Date(listsUpdatedAt).toLocaleString(),
            })}
          </p>
        ) : null}
        {(ptrRefreshing || pullVisual > 0) && (
          <motion.div
            className="pointer-events-none absolute left-0 right-0 top-0 z-10 flex justify-center text-xs text-[var(--color-muted-foreground)]"
            style={{ transform: `translateY(${Math.min(pullVisual, 48)}px)` }}
            initial={false}
            animate={{
              opacity: ptrRefreshing ? 1 : Math.min(1, 0.2 + pullVisual / 56),
            }}
            transition={{ duration: 0.12 }}
          >
            {ptrRefreshing ? t('hub.refresh.refreshing') : pullVisual > 40 ? t('hub.refresh.release') : t('hub.refresh.pull')}
          </motion.div>
        )}

        {error ? (
          <motion.div
            className="flex flex-col items-center gap-3 py-12 text-center"
            initial={reduceMotion ? false : { opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            transition={reduceMotion ? { duration: 0 } : { duration: 0.22, ease: [0.25, 0.1, 0.25, 1] }}
          >
            <p className="text-sm text-[var(--color-muted-foreground)]">{t('hub.error.body')}</p>
            <Button type="button" variant="outline" onClick={() => void refetch()}>
              {t('hub.error.retry')}
            </Button>
          </motion.div>
        ) : null}

        {!error && showSkeleton ? (
          <div
            className={cn(
              entity === 'collections' && viewMode === 'card' ? hubCardGridClass : 'flex flex-col gap-0',
            )}
          >
            {Array.from({ length: entity === 'collections' && viewMode === 'card' ? 6 : 8 }).map((_, i) =>
              entity === 'collections' && viewMode === 'card' ? (
                <div key={i} className="flex flex-col gap-2">
                  <div className="aspect-[1/1.41421356237] w-full animate-pulse rounded-lg bg-[var(--color-muted)]" />
                  <div className="h-4 w-[75%] animate-pulse rounded bg-[var(--color-muted)]" />
                </div>
              ) : (
                <div
                  key={i}
                  className={cn(
                    HUB_LIST_ROW_SHELL_CLASS,
                    HUB_LIST_ROW_INSET_LAST_CLASS,
                    entity === 'collections' && viewMode !== 'card' ? undefined : HUB_LIST_ROW_BORDER_CLASS,
                  )}
                >
                  {entity === 'collections' && viewMode !== 'card' ? (
                    <div className={cn(HUB_LIST_AVATAR_CLASS, 'animate-pulse border-0')} />
                  ) : null}
                  <div
                    className={cn(
                      entity === 'collections' && viewMode !== 'card'
                        ? HUB_LIST_ROW_TEXT_COLUMN_CLASS
                        : 'flex flex-1 flex-col gap-1.5 py-0.5',
                      entity === 'collections' && viewMode !== 'card' ? undefined : HUB_LIST_ROW_BORDER_CLASS,
                    )}
                  >
                    <div className="h-[1.0625rem] w-2/3 animate-pulse rounded bg-[var(--color-muted)]" />
                    <div className="h-[0.9375rem] w-1/2 animate-pulse rounded bg-[var(--color-muted)]" />
                  </div>
                </div>
              ),
            )}
          </div>
        ) : null}

        {!error && !showSkeleton && items.length === 0 ? (
          <motion.div
            className="flex flex-col items-center gap-3 py-16 text-center"
            initial={reduceMotion ? false : { opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={reduceMotion ? { duration: 0 } : { duration: 0.24, ease: [0.25, 0.1, 0.25, 1] }}
          >
            {debouncedQ.trim() ? (
              <>
                <p className="text-sm text-[var(--color-muted-foreground)]">{t('hub.empty.noResults')}</p>
                <Button type="button" variant="outline" size="sm" onClick={() => setQInput('')}>
                  {t('hub.empty.clearSearch')}
                </Button>
              </>
            ) : selectedTeamId ? (
              <p className="text-sm text-[var(--color-muted-foreground)]">
                {t(`hub.empty.filtered.${entity}`)}
              </p>
            ) : !networkOnline ? (
              <p className="text-sm text-[var(--color-muted-foreground)]">{t('hub.empty.offlineNone')}</p>
            ) : (
              <p className="text-sm text-[var(--color-muted-foreground)]">{t(`hub.empty.none.${entity}`)}</p>
            )}
          </motion.div>
        ) : null}

        {!error && !showSkeleton && items.length > 0 && entity === 'collections' ? (
          <div
            className={cn(
              viewMode === 'card' ? hubCardGridClass : 'flex flex-col gap-0 pb-4',
            )}
          >
            {(items as Collection[]).map((c) =>
              viewMode === 'card' ? (
                <CollectionCard
                  key={c.id}
                  collection={c}
                  onDeleteRequest={setDeleteTarget}
                  networkOnline={networkOnline}
                />
              ) : (
                <CollectionRow
                  key={c.id}
                  collection={c}
                  onDeleteRequest={setDeleteTarget}
                  networkOnline={networkOnline}
                />
              ),
            )}
          </div>
        ) : null}

        {!error && !showSkeleton && items.length > 0 && entity === 'songs' ? (
          <div className="flex flex-col pb-4">
            {(items as Song[]).map((s) => (
              <SongRow key={s.id} song={s} onDeleteRequest={setDeleteTarget} networkOnline={networkOnline} />
            ))}
          </div>
        ) : null}

        {!error && !showSkeleton && items.length > 0 && entity === 'setlists' ? (
          <div className="flex flex-col pb-4">
            {(items as Setlist[]).map((sl) => (
              <SetlistRow
                key={sl.id}
                setlist={sl}
                onDeleteRequest={setDeleteTarget}
                networkOnline={networkOnline}
              />
            ))}
          </div>
        ) : null}

        {!error && !showSkeleton && items.length > 0 ? (
          <div className="flex flex-col items-center gap-3 py-4">
            <div ref={sentinelRef} className="h-1 w-full shrink-0" aria-hidden />
            {hasNextPage ? (
              <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={isFetchingNextPage}
                onClick={() => void fetchNextPage()}
              >
                {isFetchingNextPage ? t('common.load') : t('hub.loadMore')}
              </Button>
            ) : null}
          </div>
        ) : null}
      </div>

      <AlertDialog open={deleteTarget != null} onOpenChange={(o) => !o && setDeleteTarget(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('hub.delete.title')}</AlertDialogTitle>
            <AlertDialogDescription>
              {deleteBlocked
                ? t('hub.delete.collectionNotEmptyBody', { name: deleteTarget?.label ?? '' })
                : t('hub.delete.body', { name: deleteTarget?.label ?? '' })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('hub.delete.cancel')}</AlertDialogCancel>
            {!deleteBlocked ? (
              <Button
                type="button"
                variant="destructive"
                disabled={deleteMutation.isPending || !networkOnline}
                onClick={() => {
                  if (!deleteTarget) return
                  void deleteMutation
                    .mutateAsync(deleteTarget.id)
                    .then(() => setDeleteTarget(null))
                    .catch((e: unknown) => {
                      if (e instanceof HubDeleteConflictError && e.code === 'collection_not_empty') {
                        toast.error(t('hub.delete.collectionNotEmpty'))
                        return
                      }
                      const msg = e instanceof Error ? e.message : ''
                      toast.error(msg || t('hub.delete.failed'))
                    })
                }}
              >
                {t('hub.delete.confirm')}
              </Button>
            ) : null}
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}

type DeleteTarget = { id: string; label: string; songCount?: number }
type DeleteReq = (target: DeleteTarget) => void

/** Primary tap / Enter opens `/player`. */
function useHubListItemPlayerTap(entity: HubEntity, itemId: string) {
  const navigate = useNavigate()
  const playType = hubEntityToPlayerType(entity)

  const onClick = useCallback(() => {
    void navigate({
      to: '/player',
      search: buildPlayerSearch({
        type: playType,
        id: itemId,
        mode: readPlayerDefaultMode(),
      }),
    })
  }, [navigate, playType, itemId])

  const onKeyDown = useCallback(
    (e: KeyboardEvent<HTMLElement>) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault()
        onClick()
      }
    },
    [onClick],
  )

  return { onClick, onKeyDown }
}

function HubActionItem({
  children,
  disabled,
  title,
  destructive,
  onSelect,
  onHoverChange,
}: {
  children: ReactNode
  disabled?: boolean
  title?: string
  destructive?: boolean
  onSelect?: () => void
  onHoverChange?: (hot: boolean) => void
}) {
  return (
    <Dialog.Close asChild>
      <button
        type="button"
        role="menuitem"
        disabled={disabled}
        data-disabled={disabled ? 'true' : undefined}
        title={title}
        className={cn(
          'relative flex w-full cursor-default select-none items-center gap-2 rounded-sm px-2 py-2 text-left text-sm outline-none',
          'hover:bg-[var(--color-muted)] focus:bg-[var(--color-muted)]',
          'disabled:pointer-events-none disabled:opacity-50',
          destructive && 'text-[var(--color-danger)] focus:text-[var(--color-danger)]',
        )}
        onClick={() => {
          if (disabled) return
          onSelect?.()
        }}
        onMouseEnter={() => onHoverChange?.(true)}
        onMouseLeave={() => onHoverChange?.(false)}
        onFocus={() => onHoverChange?.(true)}
        onBlur={() => onHoverChange?.(false)}
      >
        {children}
      </button>
    </Dialog.Close>
  )
}

function HubActionSeparator() {
  return <div className="my-1 h-px bg-[var(--color-border)]" role="separator" />
}

const actionIconClass = 'shrink-0 text-[var(--color-foreground)]'

type HubActionHot =
  | 'edit'
  | 'showSheets'
  | 'controlAvSlides'
  | 'saveOffline'
  | 'removeOffline'
  | 'duplicate'
  | 'addToSetlist'
  | 'exportChordpro'
  | 'exportWorshipPro'
  | 'exportSongBeamer'
  | 'exportProPresenter'
  | 'exportPdf'
  | 'delete'

function HubItemActionsMenu({
  entity,
  itemId,
  itemLabel,
  itemSongCount,
  onDeleteRequest,
  networkOnline,
  hubSong,
  variant = 'row',
}: {
  entity: HubEntity
  itemId: string
  itemLabel: string
  itemSongCount?: number
  onDeleteRequest: DeleteReq
  networkOnline: boolean
  /** When set (songs hub), enables “Add to setlist”. */
  hubSong?: Song
  variant?: 'row' | 'card'
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const chordFormat = useChordFormatPreference()
  const hideChords = useHideChordsPreference()
  const [menuHot, setMenuHot] = useState(false)
  const [closeHot, setCloseHot] = useState(false)
  const [itemHot, setItemHot] = useState<HubActionHot | null>(null)
  const [addToSetlistOpen, setAddToSetlistOpen] = useState(false)
  const hover = (key: HubActionHot) => (hot: boolean) => setItemHot(hot ? key : null)

  const playType = hubEntityToPlayerType(entity)
  const [playerCached, setPlayerCached] = useState(false)
  useEffect(() => {
    let cancelled = false
    void import('@/lib/offline/player-mirror-cache').then(({ isPlayerMirrored }) =>
      isPlayerMirrored(playType, itemId).then((v) => {
        if (!cancelled) setPlayerCached(v)
      }),
    )
    return () => {
      cancelled = true
    }
  }, [playType, itemId])
  const showAddToSetlist = Boolean(
    entity === 'songs' && hubSong && !hubSong.not_a_song,
  )
  const showSongExport = Boolean(entity === 'songs' && hubSong)
  const showOrderedExport = entity === 'setlists' || entity === 'collections'
  const showDuplicate = showOrderedExport
  const titleSuffix = t('collections.hub.duplicateTitleSuffix')
  const hubExportPdfHint = useMemo(
    () =>
      exportPdfHintTitle(
        t('hub.actions.exportPdfHint'),
        t('hub.actions.exportPdfHintSafariHeaders'),
      ),
    [t],
  )

  const onDuplicate = useCallback(async () => {
    const toastId = toast.loading(t('hub.actions.duplicate'))
    try {
      const created =
        entity === 'setlists'
          ? await duplicateSetlist(queryClient, itemId, titleSuffix)
          : await duplicateCollection(queryClient, itemId, titleSuffix)
      toast.dismiss(toastId)
      toast.success(t('hub.actions.duplicateSuccess', { title: created.title }))
      void queryClient.invalidateQueries({ queryKey: hubListRootKey })
      if (entity === 'setlists') {
        void navigate({
          to: '/setlists/$setlistId',
          params: { setlistId: created.id },
          search: emptyEditorReturnSearch(),
        })
      } else {
        void navigate({
          to: '/collections/$collectionId',
          params: { collectionId: created.id },
          search: emptyEditorReturnSearch(),
        })
      }
    } catch (e) {
      toast.dismiss(toastId)
      const detail = e instanceof Error ? e.message : String(e)
      const failedKey = 'common.duplicateFailed'
      toast.error(t(failedKey, {
        entity: t(entity === 'collections' ? 'common.entityCollection' : 'common.entitySetlist'),
      }), { description: detail })
      console.error(`${entity} duplicate failed`, e)
    }
  }, [entity, itemId, navigate, queryClient, t, titleSuffix])

  const onOrderedExport = useCallback(
    async (kind: SongExportKind) => {
      const toastId = toast.loading(t('hub.actions.exportPreparing'))
      try {
        if (entity === 'setlists') {
          await runSetlistExport(queryClient, itemId, kind, chordFormat, hideChords)
        } else if (entity === 'collections') {
          await runCollectionExport(queryClient, itemId, kind, chordFormat, hideChords)
        }
        toast.dismiss(toastId)
      } catch (e) {
        toast.dismiss(toastId)
        const detail = e instanceof Error ? e.message : String(e)
        const failedKey =
          entity === 'collections'
            ? 'hub.actions.exportCollectionFailed'
            : 'hub.actions.exportSetlistFailed'
        toast.error(t(failedKey), { description: detail })
        console.error(`${entity} export failed`, e)
      }
    },
    [chordFormat, entity, hideChords, itemId, queryClient, t],
  )

  const onSongExport = useCallback(
    async (kind: SongExportKind) => {
      if (!hubSong) return
      const toastId = toast.loading(t('hub.actions.exportPreparing'))
      try {
        await runSongExport(hubSong.data as Record<string, unknown>, kind, chordFormat, undefined, hideChords)
        toast.dismiss(toastId)
      } catch (e) {
        toast.dismiss(toastId)
        const detail = e instanceof Error ? e.message : String(e)
        toast.error(t('hub.actions.exportFailed'), { description: detail })
        console.error('Song export failed', e)
      }
    },
    [chordFormat, hideChords, hubSong, t],
  )

  const onSaveOffline = useCallback(async () => {
    if (!networkOnline) return
    const toastId = toast.loading(t('hub.actions.saveOffline'))
    const result = await downloadPlayerForOffline(playType, itemId, { title: itemLabel })
    toast.dismiss(toastId)
    if ('ok' in result && result.ok) {
      setPlayerCached(true)
      if (result.evicted) {
        toast.success(t('hub.actions.saveOfflineSuccess'), {
          description: t('offlinePlayer.storageEvicted'),
        })
      } else {
        toast.success(t('hub.actions.saveOfflineSuccess'))
      }
    } else if ('error' in result && result.error === 'offline') {
      toast.info(t('hub.refresh.offlineBlocked'))
    } else {
      toast.error(t('hub.actions.saveOfflineFailed'), {
        description: 'message' in result ? result.message : undefined,
      })
    }
  }, [itemId, itemLabel, networkOnline, playType, t])

  const onRemoveOffline = useCallback(async () => {
    await removeOfflinePlayerCopy(playType, itemId)
    setPlayerCached(false)
    toast.success(t('hub.actions.removeOfflineSuccess'))
  }, [itemId, playType, t])

  const [open, setOpen] = useState(false)
  const shouldReduceMotion = useReducedMotion()
  const [dragOffset, setDragOffset] = useState(0)
  const [isDragging, setIsDragging] = useState(false)
  const pointerStartX = useRef<number | null>(null)
  const pointerStartY = useRef<number | null>(null)
  const dragSessionActive = useRef(false)
  const dragOffsetRef = useRef(0)

  const resetDrawerDrag = useCallback(() => {
    dragSessionActive.current = false
    pointerStartX.current = null
    pointerStartY.current = null
    dragOffsetRef.current = 0
    setIsDragging(false)
    setDragOffset(0)
  }, [])

  const onDrawerOpenChange = useCallback(
    (next: boolean) => {
      if (!next) resetDrawerDrag()
      setOpen(next)
    },
    [resetDrawerDrag],
  )

  return (
    <>
      <Button
        type="button"
        variant="ghost"
        size="icon"
        className={
          variant === 'card'
            ? 'size-8 rounded-full bg-[var(--color-surface)]/80 text-[var(--color-foreground)] shadow-sm backdrop-blur-sm hover:bg-[var(--color-surface)]'
            : 'size-8 shrink-0 text-[var(--color-muted-foreground)]'
        }
        aria-label={t('hub.actions.menuAria', { title: itemLabel })}
        onClick={() => onDrawerOpenChange(true)}
        onMouseEnter={() => setMenuHot(true)}
        onMouseLeave={() => setMenuHot(false)}
        onFocus={() => setMenuHot(true)}
        onBlur={() => setMenuHot(false)}
      >
        <EllipsisIcon isHovered={menuHot} size={16} className="shrink-0" />
      </Button>
      <Dialog.Root open={open} onOpenChange={onDrawerOpenChange}>
        <Dialog.Portal forceMount>
          <AnimatePresence>
            {open ? (
              <>
                <Dialog.Overlay forceMount asChild>
                  <motion.div
                    className="fixed inset-0 z-50 bg-black/40"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    transition={{ duration: shouldReduceMotion ? 0 : 0.18 }}
                  />
                </Dialog.Overlay>
                <Dialog.Content forceMount asChild aria-describedby={undefined}>
                  <motion.div
                    className={cn(
                      'fixed inset-y-0 right-0 z-50 flex w-[min(22rem,90vw)] flex-row border-l border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-foreground)] shadow-[var(--shadow-elevated)]',
                      'rounded-l-2xl pt-[env(safe-area-inset-top,0px)] pb-[env(safe-area-inset-bottom,0px)]',
                    )}
                    initial={{ x: shouldReduceMotion ? 0 : '100%' }}
                    animate={isDragging ? { x: dragOffset } : { x: 0 }}
                    exit={{ x: shouldReduceMotion ? 0 : '100%' }}
                    transition={
                      isDragging
                        ? { duration: 0 }
                        : { type: 'spring', stiffness: 420, damping: 36, mass: 0.9 }
                    }
                    onPointerDown={(event) => {
                      pointerStartX.current = event.clientX
                      pointerStartY.current = event.clientY
                    }}
                    onPointerMove={(event) => {
                      if (pointerStartX.current == null || pointerStartY.current == null) return
                      const dx = event.clientX - pointerStartX.current
                      const dy = event.clientY - pointerStartY.current
                      if (!dragSessionActive.current) {
                        if (Math.hypot(dx, dy) < 8) return
                        if (dx < 10 || Math.abs(dy) >= dx) {
                          pointerStartX.current = null
                          pointerStartY.current = null
                          return
                        }
                        dragSessionActive.current = true
                        setIsDragging(true)
                        try {
                          event.currentTarget.setPointerCapture(event.pointerId)
                        } catch {
                          /* capture may fail if the pointer already released */
                        }
                      }
                      const next = Math.max(0, dx)
                      dragOffsetRef.current = next
                      setDragOffset(next)
                    }}
                    onPointerUp={() => {
                      if (!dragSessionActive.current) {
                        pointerStartX.current = null
                        pointerStartY.current = null
                        return
                      }
                      const offset = dragOffsetRef.current
                      resetDrawerDrag()
                      if (offset > 90) onDrawerOpenChange(false)
                    }}
                    onPointerCancel={() => {
                      resetDrawerDrag()
                    }}
                  >
                    <div
                      className="flex w-8 shrink-0 items-center justify-center"
                      aria-hidden
                    >
                      <div className="h-12 w-1.5 rounded-full bg-[var(--color-muted)]" />
                    </div>
                    <div className="flex min-h-0 min-w-0 flex-1 flex-col">
                    <div className="flex items-center gap-2 border-b border-[var(--color-border)] py-3 pr-3">
                      <Dialog.Title className="min-w-0 flex-1 truncate text-base font-semibold">
                        {itemLabel}
                      </Dialog.Title>
                      <Dialog.Close asChild>
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon"
                          className="size-8 shrink-0"
                          aria-label={t('hub.actions.closeAria')}
                          onMouseEnter={() => setCloseHot(true)}
                          onMouseLeave={() => setCloseHot(false)}
                          onFocus={() => setCloseHot(true)}
                          onBlur={() => setCloseHot(false)}
                        >
                          <XIcon isHovered={closeHot} size={16} className="shrink-0" />
                        </Button>
                      </Dialog.Close>
                    </div>
                    <nav
                      className="flex min-h-0 flex-1 flex-col overflow-y-auto p-3"
                      role="menu"
                      aria-label={t('hub.actions.menuAria', { title: itemLabel })}
                    >
            <div role="group" aria-label={t('hub.actions.general')}>
              <div className="px-2 pb-1 text-xs font-semibold text-[var(--color-muted-foreground)]">
                {t('hub.actions.general')}
              </div>
            <HubActionItem
              onSelect={() => {
                if (entity === 'setlists') {
                  void navigate({
                    to: '/setlists/$setlistId',
                    params: { setlistId: itemId },
                    search: emptyEditorReturnSearch(),
                  })
                } else if (entity === 'collections') {
                  void navigate({
                    to: '/collections/$collectionId',
                    params: { collectionId: itemId },
                    search: emptyEditorReturnSearch(),
                  })
                } else if (entity === 'songs') {
                  void navigate({
                    to: '/songs/$songId',
                    params: { songId: itemId },
                    search: emptyEditorReturnSearch(),
                  })
                } else {
                  void navigate({
                    to: '/$',
                    params: { _splat: hubEntityEditSplat(entity, itemId) },
                  })
                }
              }}
              onHoverChange={hover('edit')}
            >
              <PencilIcon isHovered={itemHot === 'edit'} size={16} className={actionIconClass} />
              {t('hub.actions.edit')}
            </HubActionItem>
            <HubActionItem
              onSelect={() => {
                void navigate({
                  to: '/player',
                  search: buildPlayerSearch({ type: playType, id: itemId, mode: 'sheet' }),
                })
              }}
              onHoverChange={hover('showSheets')}
            >
              <FileTextIcon isHovered={itemHot === 'showSheets'} size={16} className={actionIconClass} />
              {t('hub.actions.showSheets')}
            </HubActionItem>
            <HubActionItem
              onSelect={() => {
                void navigate({
                  to: '/player',
                  search: buildPlayerSearch({ type: playType, id: itemId, mode: 'av' }),
                })
              }}
              onHoverChange={hover('controlAvSlides')}
            >
              <OutputIcon isHovered={itemHot === 'controlAvSlides'} size={16} className={actionIconClass} />
              {t('hub.actions.controlAvSlides')}
            </HubActionItem>
            {playerCached ? (
              <HubActionItem onSelect={() => void onRemoveOffline()} onHoverChange={hover('removeOffline')}>
                <FolderXIcon isHovered={itemHot === 'removeOffline'} size={16} className={actionIconClass} />
                {t('hub.actions.removeOffline')}
              </HubActionItem>
            ) : (
              <HubActionItem
                disabled={!networkOnline}
                title={!networkOnline ? t('hub.createOfflineHint') : undefined}
                onSelect={() => void onSaveOffline()}
                onHoverChange={hover('saveOffline')}
              >
                <DownloadIcon isHovered={itemHot === 'saveOffline'} size={16} className={actionIconClass} />
                {t('hub.actions.saveOffline')}
              </HubActionItem>
            )}
            {showDuplicate ? (
              <HubActionItem
                disabled={!networkOnline}
                title={!networkOnline ? t('hub.actions.deleteOfflineHint') : undefined}
                onSelect={() => {
                  if (!networkOnline) return
                  void onDuplicate()
                }}
                onHoverChange={hover('duplicate')}
              >
                <CopyIcon isHovered={itemHot === 'duplicate'} size={16} className={actionIconClass} />
                {t('hub.actions.duplicate')}
              </HubActionItem>
            ) : null}
            {showAddToSetlist ? (
              <HubActionItem
                disabled={!networkOnline}
                title={!networkOnline ? t('hub.createOfflineHint') : undefined}
                onSelect={() => {
                  if (!networkOnline) return
                  setAddToSetlistOpen(true)
                }}
                onHoverChange={hover('addToSetlist')}
              >
                <ListMusicIcon isHovered={itemHot === 'addToSetlist'} size={16} className={actionIconClass} />
                {t('hub.actions.addToSetlist')}
              </HubActionItem>
            ) : null}
            </div>
            {showSongExport || showOrderedExport ? (
              <>
                <HubActionSeparator />
                <div role="group" aria-label={t('hub.actions.export')}>
                  <div className="px-2 pb-1 pt-2 text-xs font-semibold text-[var(--color-muted-foreground)]">
                    {t('hub.actions.export')}
                  </div>
                  <HubActionItem
                    onSelect={() => void (showSongExport ? onSongExport('chordpro') : onOrderedExport('chordpro'))}
                    onHoverChange={hover('exportChordpro')}
                  >
                    <FileTextIcon isHovered={itemHot === 'exportChordpro'} size={16} className={actionIconClass} />
                    {t('hub.actions.exportChordPro')}
                  </HubActionItem>
                  <HubActionItem
                    onSelect={() => void (showSongExport ? onSongExport('worshippro') : onOrderedExport('worshippro'))}
                    onHoverChange={hover('exportWorshipPro')}
                  >
                    <FileStackIcon isHovered={itemHot === 'exportWorshipPro'} size={16} className={actionIconClass} />
                    {t('hub.actions.exportWorshipPro')}
                  </HubActionItem>
                  <HubActionItem
                    onSelect={() => void (showSongExport ? onSongExport('songbeamer') : onOrderedExport('songbeamer'))}
                    onHoverChange={hover('exportSongBeamer')}
                  >
                    <ProjectorIcon isHovered={itemHot === 'exportSongBeamer'} size={16} className={actionIconClass} />
                    {t('hub.actions.exportSongBeamer')}
                  </HubActionItem>
                  <HubActionItem
                    onSelect={() => void (showSongExport ? onSongExport('propresenter') : onOrderedExport('propresenter'))}
                    onHoverChange={hover('exportProPresenter')}
                  >
                    <OutputIcon isHovered={itemHot === 'exportProPresenter'} size={16} className={actionIconClass} />
                    {t('hub.actions.exportProPresenter')}
                  </HubActionItem>
                  <HubActionItem
                    title={hubExportPdfHint}
                    onSelect={() => void (showSongExport ? onSongExport('pdf') : onOrderedExport('pdf'))}
                    onHoverChange={hover('exportPdf')}
                  >
                    <PrinterIcon isHovered={itemHot === 'exportPdf'} size={16} className={actionIconClass} />
                    {t('hub.actions.exportPdf')}
                  </HubActionItem>
                </div>
              </>
            ) : null}
            <HubActionSeparator />
            <HubActionItem
              destructive
              disabled={!networkOnline}
              title={!networkOnline ? t('hub.actions.deleteOfflineHint') : undefined}
              onSelect={() => {
                if (!networkOnline) return
                onDeleteRequest({
                  id: itemId,
                  label: itemLabel,
                  ...(itemSongCount != null ? { songCount: itemSongCount } : {}),
                })
              }}
              onHoverChange={hover('delete')}
            >
              <TrashIcon isHovered={itemHot === 'delete'} size={16} className="shrink-0" />
              {t('hub.actions.delete')}
            </HubActionItem>
                    </nav>
                    </div>
                  </motion.div>
                </Dialog.Content>
              </>
            ) : null}
          </AnimatePresence>
        </Dialog.Portal>
      </Dialog.Root>
      {showAddToSetlist && hubSong ? (
        <AddSongToSetlistDialog open={addToSetlistOpen} onOpenChange={setAddToSetlistOpen} song={hubSong} />
      ) : null}
    </>
  )
}

const CollectionCard = memo(function CollectionCard({
  collection,
  onDeleteRequest,
  networkOnline,
}: {
  collection: Collection
  onDeleteRequest: DeleteReq
  networkOnline: boolean
}) {
  const reduceMotion = useReducedMotion()
  const { onClick, onKeyDown } = useHubListItemPlayerTap('collections', collection.id)
  const { src: coverSrc, onImageError: onCoverError } = useCoverImageSrc(collection.cover)

  return (
    <div className="relative">
      <motion.div
        className="flex cursor-pointer flex-col gap-1.5 rounded-lg outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] sm:gap-2"
        onClick={onClick}
        role="button"
        tabIndex={0}
        aria-label={collection.title}
        whileTap={reduceMotion ? undefined : tapFeedback}
        transition={tapTransition}
        onKeyDown={onKeyDown}
      >
        <div className="relative aspect-[1/1.41421356237] w-full overflow-hidden rounded-lg border border-[var(--color-border)] bg-[var(--color-muted)]">
          {coverSrc ? (
            <img
              src={coverSrc}
              alt=""
              draggable={false}
              className="pointer-events-none size-full object-cover"
              loading="lazy"
              onError={onCoverError}
            />
          ) : null}
        </div>
        <p className="line-clamp-2 text-xs font-medium leading-snug text-[var(--color-foreground)] sm:text-sm xl:text-[0.6875rem] xl:leading-tight">
          {collection.title}
        </p>
      </motion.div>
      <div className="absolute right-1 top-1 z-10">
        <HubItemActionsMenu
          entity="collections"
          itemId={collection.id}
          itemLabel={collection.title}
          itemSongCount={collection.songs.length}
          onDeleteRequest={onDeleteRequest}
          networkOnline={networkOnline}
          variant="card"
        />
      </div>
    </div>
  )
})

const CollectionRow = memo(function CollectionRow({
  collection,
  onDeleteRequest,
  networkOnline,
}: {
  collection: Collection
  onDeleteRequest: DeleteReq
  networkOnline: boolean
}) {
  const { t } = useTranslation()
  const { data: user } = useSession()
  const reduceMotion = useReducedMotion()
  const { onClick, onKeyDown } = useHubListItemPlayerTap('collections', collection.id)
  const { src: coverSrc, onImageError: onCoverError } = useCoverImageSrc(collection.cover)
  const { data: ownerTeam, isPending: ownerTeamPending, isError: ownerTeamError } =
    useTeamDetail(collection.owner)

  const ownerLabel = useMemo(() => {
    if (ownerTeamPending) return null
    if (ownerTeamError || !ownerTeam) return t('setlists.editor.teamUnavailable')
    return getTeamDisplayName(ownerTeam, user?.id, t)
  }, [ownerTeam, ownerTeamError, ownerTeamPending, t, user?.id])

  const songsCount = t('hub.meta.songsCount', { count: collection.songs.length })
  const subtitle = ownerLabel ? `${songsCount}, ${ownerLabel}` : songsCount

  return (
    <div className={cn(HUB_LIST_ROW_SHELL_CLASS, HUB_LIST_ROW_INSET_LAST_CLASS, 'cursor-default')}>
      <div className={HUB_LIST_AVATAR_CLASS}>
        {coverSrc ? (
          <img
            src={coverSrc}
            alt=""
            draggable={false}
            className="pointer-events-none size-full object-cover"
            loading="lazy"
            onError={onCoverError}
          />
        ) : null}
      </div>
      <div className={cn(HUB_LIST_ROW_TEXT_COLUMN_CLASS, 'flex-row items-center gap-1')}>
        <motion.div
          className="min-w-0 flex-1 cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-primary)]"
          onClick={onClick}
          role="button"
          tabIndex={0}
          aria-label={collection.title}
          whileTap={reduceMotion ? undefined : tapFeedback}
          transition={tapTransition}
          onKeyDown={onKeyDown}
        >
          <p className={HUB_LIST_TITLE_CLASS}>{collection.title}</p>
          <p className={cn(HUB_LIST_SUBTITLE_CLASS, 'truncate')} title={subtitle}>
            {subtitle}
          </p>
        </motion.div>
        <HubItemActionsMenu
          entity="collections"
          itemId={collection.id}
          itemLabel={collection.title}
          itemSongCount={collection.songs.length}
          onDeleteRequest={onDeleteRequest}
          networkOnline={networkOnline}
        />
      </div>
    </div>
  )
})

const SongRow = memo(function SongRow({
  song,
  onDeleteRequest,
  networkOnline,
}: {
  song: Song
  onDeleteRequest: DeleteReq
  networkOnline: boolean
}) {
  const { t } = useTranslation()
  const { data: user } = useSession()
  const reduceMotion = useReducedMotion()
  const { onClick, onKeyDown } = useHubListItemPlayerTap('songs', song.id)
  const title = songTitle(song)
  const sub = songSubtitle(song, t('hub.meta.unknownArtist'))
  const { data: ownerTeam, isPending: ownerTeamPending, isError: ownerTeamError } =
    useTeamDetail(song.owner)

  const ownerLabel = useMemo(() => {
    if (ownerTeamPending) return null
    if (ownerTeamError || !ownerTeam) return t('setlists.editor.teamUnavailable')
    return getTeamDisplayName(ownerTeam, user?.id, t)
  }, [ownerTeam, ownerTeamError, ownerTeamPending, t, user?.id])

  const subtitle = ownerLabel ? `${sub}, ${ownerLabel}` : sub

  return (
    <div className={cn('flex items-center', HUB_LIST_ROW_BORDER_CLASS)}>
      <motion.div
        className={cn(HUB_LIST_ROW_SHELL_CLASS, 'min-w-0 flex-1')}
        onClick={onClick}
        role="button"
        tabIndex={0}
        aria-label={title}
        whileTap={reduceMotion ? undefined : tapFeedback}
        transition={tapTransition}
        onKeyDown={onKeyDown}
      >
        <div className="flex min-w-0 flex-1 flex-col justify-center py-0.5">
          <p className={HUB_LIST_TITLE_CLASS}>{title}</p>
          <p className={cn(HUB_LIST_SUBTITLE_CLASS, 'truncate')} title={subtitle}>
            {subtitle}
          </p>
        </div>
      </motion.div>
      <HubItemActionsMenu
        entity="songs"
        itemId={song.id}
        itemLabel={title}
        onDeleteRequest={onDeleteRequest}
        networkOnline={networkOnline}
        hubSong={song}
      />
    </div>
  )
})

const SetlistRow = memo(function SetlistRow({
  setlist,
  onDeleteRequest,
  networkOnline,
}: {
  setlist: Setlist
  onDeleteRequest: DeleteReq
  networkOnline: boolean
}) {
  const { t } = useTranslation()
  const { data: user } = useSession()
  const reduceMotion = useReducedMotion()
  const { onClick, onKeyDown } = useHubListItemPlayerTap('setlists', setlist.id)
  const { data: ownerTeam, isPending: ownerTeamPending, isError: ownerTeamError } =
    useTeamDetail(setlist.owner)

  const ownerLabel = useMemo(() => {
    if (ownerTeamPending) return null
    if (ownerTeamError || !ownerTeam) return t('setlists.editor.teamUnavailable')
    return getTeamDisplayName(ownerTeam, user?.id, t)
  }, [ownerTeam, ownerTeamError, ownerTeamPending, t, user?.id])

  return (
    <div className={cn('flex items-center', HUB_LIST_ROW_BORDER_CLASS)}>
      <motion.div
        className={cn(HUB_LIST_ROW_SHELL_CLASS, 'min-w-0 flex-1')}
        onClick={onClick}
        role="button"
        tabIndex={0}
        aria-label={setlist.title}
        whileTap={reduceMotion ? undefined : tapFeedback}
        transition={tapTransition}
        onKeyDown={onKeyDown}
      >
        <div className="min-w-0 flex-1 flex flex-col justify-center py-0.5">
          <p className={HUB_LIST_TITLE_CLASS}>{setlist.title}</p>
          <p className={cn(HUB_LIST_SUBTITLE_CLASS, 'truncate')}>
            <SetlistItemCounts items={setlist.items} />
            {ownerLabel ? `, ${ownerLabel}` : null}
          </p>
        </div>
      </motion.div>
      <HubItemActionsMenu
        entity="setlists"
        itemId={setlist.id}
        itemLabel={setlist.title}
        onDeleteRequest={onDeleteRequest}
        networkOnline={networkOnline}
      />
    </div>
  )
})
