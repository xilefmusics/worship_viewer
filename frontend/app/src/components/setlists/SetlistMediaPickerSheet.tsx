import * as Dialog from '@radix-ui/react-dialog'
import { useInfiniteQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { AnimatePresence, motion, useReducedMotion } from 'motion/react'
import { useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { createUploadedMedia } from '@/api/media-upload'
import {
  fetchMediaPage,
  mediaDetailKey,
  mediaListKey,
  mediaListRootKey,
  type Media,
} from '@/api/media'
import { PlusIcon } from '@/components/icons/lucide-animated/plus-icon'
import { SettingsIcon } from '@/components/icons/lucide-animated/settings-icon'
import { CreateMediaDialog } from '@/components/media/CreateMediaDialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { useDebounced } from '@/hooks/useSongPickerQuery'
import { getNextPageIndex } from '@/lib/list-pagination'
import { mediaDisplayKind, sniffAssetUploadKind } from '@/lib/media-display'
import { cn } from '@/lib/utils'

export function SetlistMediaPickerSheet({
  open,
  onOpenChange,
  blockedAdd = false,
  duplicateCountFor,
  onPickMedia,
  defaultOwner,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  blockedAdd?: boolean
  duplicateCountFor: (mediaId: string) => number
  onPickMedia: (media: Media) => void
  defaultOwner?: string
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const shouldReduceMotion = useReducedMotion()
  const [q, setQ] = useState('')
  const [createOpen, setCreateOpen] = useState(false)
  const [quickUploadProgress, setQuickUploadProgress] = useState<number | null>(null)
  const [isDropActive, setIsDropActive] = useState(false)
  const [dragOffset, setDragOffset] = useState(0)
  const [isDragging, setIsDragging] = useState(false)
  const pointerStartY = useRef<number | null>(null)
  const debouncedQ = useDebounced(300, q)
  const query = useInfiniteQuery({
    queryKey: [...mediaListKey(debouncedQ, null), 'picker'],
    enabled: open,
    initialPageParam: 0,
    queryFn: ({ pageParam, signal }) =>
      fetchMediaPage(queryClient, {
        page: pageParam as number,
        q: debouncedQ,
        teamId: null,
        signal,
      }),
    getNextPageParam: (_last, all) => getNextPageIndex(all),
  })
  const items = useMemo(
    () => query.data?.pages.flatMap((page) => page.items) ?? [],
    [query.data?.pages],
  )
  const quickUpload = useMutation({
    mutationFn: async (files: File[]) => {
      if (!defaultOwner) throw new Error(t('media.validation.noWritableTeam'))
      const uploadKinds = files.map(sniffAssetUploadKind)
      if (files.length === 0 || uploadKinds.some((kind) => kind == null)) {
        throw new Error(t('setlists.editor.mediaQuickUploadUnsupported'))
      }
      const isDeck = uploadKinds.every(
        (kind) => kind === 'image' || kind === 'pdf' || kind === 'svg',
      )
      const createKind = isDeck
        ? 'slide_deck'
        : files.length === 1 && (uploadKinds[0] === 'video' || uploadKinds[0] === 'audio')
          ? uploadKinds[0]
          : null
      if (!createKind) throw new Error(t('setlists.editor.mediaQuickUploadUnsupported'))

      const title = files[0]?.name.replace(/\.[^.]+$/, '').trim() || t('setlists.editor.addMediaTitle')
      setQuickUploadProgress(0)
      return createUploadedMedia({
        kind: createKind,
        title,
        owner: defaultOwner,
        files,
        onProgress: setQuickUploadProgress,
      })
    },
    onSuccess: (created) => {
      void queryClient.invalidateQueries({ queryKey: mediaListRootKey })
      queryClient.setQueryData(mediaDetailKey(created.id), created)
      setQuickUploadProgress(null)
      onPickMedia(created)
      setQ('')
      onOpenChange(false)
    },
    onError: () => setQuickUploadProgress(null),
  })

  const waitingForCreatedMedia = quickUpload.isPending

  const close = () => {
    setQ('')
    onOpenChange(false)
  }

  return (
    <Dialog.Root open={open} onOpenChange={(next) => (next ? onOpenChange(true) : close())}>
      <Dialog.Portal forceMount>
        <AnimatePresence>
          {open ? (
            <>
              <Dialog.Overlay forceMount asChild>
                <motion.div
                  className="fixed inset-0 z-[60] bg-black/40"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  transition={{ duration: shouldReduceMotion ? 0 : 0.18 }}
                />
              </Dialog.Overlay>
              <Dialog.Content forceMount asChild>
                <motion.div
                  className={cn(
                    'fixed inset-x-0 bottom-0 z-[61] flex max-h-[min(32rem,88dvh)] w-full flex-col gap-3 rounded-t-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 text-[var(--color-foreground)] shadow-[var(--shadow-elevated)]',
                  )}
                  initial={{ y: shouldReduceMotion ? 0 : '100%' }}
                  animate={isDragging ? { y: dragOffset } : { y: 0 }}
                  exit={{ y: shouldReduceMotion ? 0 : '100%' }}
                  transition={
                    isDragging
                      ? { duration: 0 }
                      : { type: 'spring', stiffness: 420, damping: 36, mass: 0.9 }
                  }
                >
                  <div
                    className="mx-auto h-1.5 w-12 shrink-0 rounded-full bg-[var(--color-muted)]"
                    style={{ touchAction: 'none' }}
                    onPointerDown={(event) => {
                      event.currentTarget.setPointerCapture(event.pointerId)
                      pointerStartY.current = event.clientY
                      setIsDragging(true)
                      setDragOffset(0)
                    }}
                    onPointerMove={(event) => {
                      if (!isDragging || pointerStartY.current === null) return
                      setDragOffset(Math.max(0, event.clientY - pointerStartY.current))
                    }}
                    onPointerUp={() => {
                      if (!isDragging) return
                      setIsDragging(false)
                      pointerStartY.current = null
                      if (dragOffset > 90) close()
                      setDragOffset(0)
                    }}
                    onPointerCancel={() => {
                      setIsDragging(false)
                      pointerStartY.current = null
                      setDragOffset(0)
                    }}
                  />
                  <Dialog.Title className="text-base font-semibold">
                    {t('setlists.editor.addMediaTitle')}
                  </Dialog.Title>
                  <Input
                    value={q}
                    onChange={(event) => setQ(event.target.value)}
                    placeholder={t('setlists.editor.mediaPickerSearchPlaceholder')}
                    aria-label={t('setlists.editor.mediaPickerSearchAria')}
                    autoComplete="off"
                  />
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
                        <span className="block text-xs text-[var(--color-muted-foreground)]">{t(`media.kinds.${kind}`)}</span>
                      </span>
                      {duplicateCount > 0 ? <span className="shrink-0 text-[0.65rem] uppercase text-[var(--color-muted-foreground)]">{t('common.duplicateBadge', { container: t('common.containerSetlist'), count: duplicateCount })}</span> : null}
                    </button>
                  </li>
                )
              })}
              <li className="flex items-stretch gap-2 py-1">
                <label
                  className={cn(
                    'flex min-h-16 min-w-0 flex-1 cursor-pointer items-center rounded-lg border border-dashed border-[var(--color-border)] px-4 py-3 transition-colors hover:bg-[var(--color-muted)]',
                    isDropActive && 'border-[var(--color-primary)] bg-[var(--color-primary)]/5',
                    (blockedAdd || waitingForCreatedMedia || !defaultOwner) &&
                      'cursor-not-allowed opacity-55',
                  )}
                  onDragEnter={(event) => {
                    event.preventDefault()
                    if (!blockedAdd && !waitingForCreatedMedia && defaultOwner) setIsDropActive(true)
                  }}
                  onDragOver={(event) => {
                    event.preventDefault()
                    event.dataTransfer.dropEffect = 'copy'
                  }}
                  onDragLeave={(event) => {
                    if (!event.currentTarget.contains(event.relatedTarget as Node | null)) {
                      setIsDropActive(false)
                    }
                  }}
                  onDrop={(event) => {
                    event.preventDefault()
                    setIsDropActive(false)
                    if (blockedAdd || waitingForCreatedMedia || !defaultOwner) return
                    quickUpload.mutate([...event.dataTransfer.files])
                  }}
                >
                  <input
                    type="file"
                    className="sr-only"
                    multiple
                    accept="image/png,image/jpeg,image/svg+xml,application/pdf,video/*,audio/*,.png,.jpg,.jpeg,.svg,.pdf"
                    disabled={blockedAdd || waitingForCreatedMedia || !defaultOwner}
                    aria-label={t('setlists.editor.mediaQuickUploadAria')}
                    onChange={(event) => {
                      const files = [...(event.target.files ?? [])]
                      if (files.length > 0) quickUpload.mutate(files)
                      event.target.value = ''
                    }}
                  />
                  <span className="min-w-0">
                    <span className="block text-sm font-medium">
                      {quickUpload.isPending
                        ? t('media.upload.progress', {
                            percent: Math.round((quickUploadProgress ?? 0) * 100),
                          })
                        : t('setlists.editor.mediaQuickUploadTitle')}
                    </span>
                    <span className="mt-0.5 block truncate text-xs text-[var(--color-muted-foreground)]">
                      {t('setlists.editor.mediaQuickUploadHint')}
                    </span>
                  </span>
                </label>
                <Button
                  type="button"
                  variant="outline"
                  size="icon"
                  disabled={blockedAdd || waitingForCreatedMedia}
                  title={t('setlists.editor.createMedia')}
                  aria-label={t('setlists.editor.createMedia')}
                  onClick={() => setCreateOpen(true)}
                >
                  <PlusIcon size={18} />
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  size="icon"
                  title={t('setlists.editor.manageMedia')}
                  aria-label={t('setlists.editor.manageMedia')}
                  onClick={() => {
                    close()
                    void navigate({ to: '/media' })
                  }}
                >
                  <SettingsIcon size={18} />
                </Button>
              </li>
              {quickUpload.isError ? (
                <li
                  role="alert"
                  className="px-3 py-1 text-sm text-[var(--color-destructive)]"
                >
                  {quickUpload.error.message}
                </li>
              ) : null}
            </ul>
            {query.hasNextPage ? <div className="flex justify-center pb-3"><Button type="button" size="sm" variant="outline" disabled={query.isFetchingNextPage} onClick={() => void query.fetchNextPage()}>{query.isFetchingNextPage ? t('common.load') : t('hub.loadMore')}</Button></div> : null}
          </div>
                  <div className="flex justify-end pt-1">
                    <Button type="button" variant="outline" onClick={close}>
                      {t('common.cancel')}
                    </Button>
                  </div>
                </motion.div>
              </Dialog.Content>
            </>
          ) : null}
        </AnimatePresence>
      </Dialog.Portal>
      <CreateMediaDialog
        open={createOpen}
        onOpenChange={setCreateOpen}
        defaultOwner={defaultOwner}
        elevated
        onCreated={(id, media) => {
          queryClient.setQueryData(mediaDetailKey(id), media)
          onPickMedia(media)
          setQ('')
          setCreateOpen(false)
          onOpenChange(false)
        }}
      />
    </Dialog.Root>
  )
}
