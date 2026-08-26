import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

import { uploadMediaSource } from '@/api/media-upload'
import {
  cancelMediaProcessing,
  fetchMedia,
  mediaDetailKey,
  mediaListRootKey,
  updateMedia,
} from '@/api/media'
import { mediaAssetDataUrl } from '@/api/media-upload'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { useOnline } from '@/hooks/use-online'
import { useTeamDetail } from '@/hooks/useTeamDetail'
import { useWritableTeams } from '@/hooks/useWritableTeams'
import {
  formatMediaDuration,
  hasReplacementFailure,
  isProcessingActive,
  isReadyUploaded,
  isUploadedDisplayKind,
  isUrlMediaKind,
  isValidUrlMediaInput,
  mediaCanonicalUrl,
  mediaDisplayKind,
  mediaDisplayStatus,
  type UrlMediaKind,
  urlContent,
} from '@/lib/media-display'
import { canEditTeamLibrary } from '@/lib/team-permissions'

export function MediaEditorScreen({ mediaId }: { mediaId: string }) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const online = useOnline()
  const fileInputRef = useRef<HTMLInputElement>(null)
  const detailQuery = useQuery({
    queryKey: mediaDetailKey(mediaId),
    queryFn: ({ signal }) => fetchMedia(queryClient, mediaId, signal),
    refetchInterval: (query) => {
      const media = query.state.data
      return media && isProcessingActive(media) ? 2000 : false
    },
  })
  const media = detailQuery.data
  const ownerTeam = useTeamDetail(media?.owner ?? '', { enabled: Boolean(media?.owner) })
  const { user } = useWritableTeams(`mediaEditor:${mediaId}`, Boolean(media))
  const canEdit = Boolean(media && ownerTeam.data && user?.id && canEditTeamLibrary(ownerTeam.data, user.id))
  const [title, setTitle] = useState('')
  const [kind, setKind] = useState<UrlMediaKind>('youtube')
  const [url, setUrl] = useState('')
  const [owner, setOwner] = useState('')
  const [error, setError] = useState('')
  const [uploadProgress, setUploadProgress] = useState<number | null>(null)

  useEffect(() => {
    if (!media) return
    // eslint-disable-next-line react-hooks/set-state-in-effect -- reset the editor draft from the fetched canonical resource
    setTitle(media.title)
    setOwner(media.owner)
    const displayKind = mediaDisplayKind(media)
    if (isUrlMediaKind(displayKind)) {
      setKind(displayKind)
      setUrl(mediaCanonicalUrl(media) ?? '')
    }
  }, [media])

  const displayKind = media ? mediaDisplayKind(media) : 'unknown'
  const displayStatus = media ? mediaDisplayStatus(media) : 'unknown'
  const uploaded = media ? isUploadedDisplayKind(displayKind) : false
  const editableUrl = media ? displayStatus === 'ready' && isUrlMediaKind(displayKind) : false
  const processingActive = media ? isProcessingActive(media) : false
  const initialFailed = media?.status === 'failed'
  const replacementFailed = media ? hasReplacementFailure(media) : false
  const readyUploaded = media ? isReadyUploaded(media) : false

  const saveMutation = useMutation({
    mutationFn: async () => {
      if (!canEdit) throw new Error(t('media.validation.notEditable'))
      if (uploaded) {
        return updateMedia(queryClient, mediaId, { title: title.trim(), owner })
      }
      if (!editableUrl) throw new Error(t('media.validation.notEditable'))
      if (!isValidUrlMediaInput(kind, url)) throw new Error(t('media.validation.invalidUrl'))
      return updateMedia(queryClient, mediaId, {
        title: title.trim(),
        owner,
        content: urlContent(kind, url.trim()),
      })
    },
    onSuccess: (saved) => {
      queryClient.setQueryData(mediaDetailKey(mediaId), saved)
      void queryClient.invalidateQueries({ queryKey: mediaListRootKey })
      toast.success(t('media.editor.saved'))
    },
    onError: (cause: Error) => setError(cause.message),
  })

  const uploadMutation = useMutation({
    mutationFn: async (file: File) => {
      if (!media || !isUploadedDisplayKind(displayKind)) throw new Error(t('media.validation.notEditable'))
      const uploadKind = displayKind === 'video' ? 'video' : 'audio'
      setUploadProgress(0)
      await uploadMediaSource({
        mediaId,
        kind: uploadKind,
        file,
        onProgress: (ratio) => setUploadProgress(ratio),
      })
      return fetchMedia(queryClient, mediaId)
    },
    onSuccess: (saved) => {
      setUploadProgress(null)
      queryClient.setQueryData(mediaDetailKey(mediaId), saved)
      void queryClient.invalidateQueries({ queryKey: mediaListRootKey })
    },
    onError: (cause: Error) => {
      setUploadProgress(null)
      setError(cause.message)
    },
  })

  const cancelMutation = useMutation({
    mutationFn: () => cancelMediaProcessing(queryClient, mediaId),
    onSuccess: (saved) => {
      queryClient.setQueryData(mediaDetailKey(mediaId), saved)
      toast.success(t('media.upload.cancelled'))
    },
    onError: (cause: Error) => toast.error(cause.message),
  })

  if (detailQuery.isPending) {
    return <div className="mx-auto w-full max-w-2xl py-12 text-center text-sm text-[var(--color-muted-foreground)]" role="status">{t('common.load')}</div>
  }
  if (detailQuery.error || !media) {
    return <div className="mx-auto flex w-full max-w-2xl flex-col items-center gap-3 py-12 text-center"><p className="text-sm text-[var(--color-muted-foreground)]">{t('media.editor.loadFailed')}</p><Button variant="outline" onClick={() => void detailQuery.refetch()}>{t('hub.error.retry')}</Button></div>
  }

  const canonicalUrl = mediaCanonicalUrl(media)
  const previewBlobId =
    media.content?.type === 'video' || media.content?.type === 'audio'
      ? media.content.blob_id
      : null
  const previewUrl = previewBlobId ? mediaAssetDataUrl(mediaId, previewBlobId) : null

  return (
    <section className="mx-auto grid w-full max-w-2xl gap-6 pb-8" aria-labelledby="media-editor-heading">
      <div className="grid gap-1">
        <h1 id="media-editor-heading" className="text-xl font-semibold">{media.title}</h1>
        <p className="text-sm text-[var(--color-muted-foreground)]">{t(`media.states.${displayStatus}`)} · {t(`media.kinds.${displayKind}`)}</p>
      </div>

      {initialFailed ? (
        <div role="alert" className="rounded-lg border border-[var(--color-destructive)]/40 bg-[var(--color-destructive)]/5 p-3 text-sm">
          <p className="font-medium">{t('media.editor.failedTitle')}</p>
          <p className="text-[var(--color-muted-foreground)]">{media.pending_revision?.processing_error?.detail ?? t('media.editor.failedBody')}</p>
        </div>
      ) : null}

      {replacementFailed ? (
        <div role="alert" className="rounded-lg border border-[var(--color-destructive)]/40 bg-[var(--color-destructive)]/5 p-3 text-sm">
          <p className="font-medium">{t('media.editor.replacementFailedTitle')}</p>
          <p className="text-[var(--color-muted-foreground)]">{media.pending_revision?.processing_error?.detail ?? t('media.editor.replacementFailedBody')}</p>
        </div>
      ) : null}

      {processingActive ? (
        <div role="status" className="rounded-lg border border-[var(--color-border)] bg-[var(--color-muted)]/30 p-3 text-sm">
          {readyUploaded ? t('media.editor.replacementProcessingBody') : t('media.editor.processingBody')}
        </div>
      ) : null}

      {uploaded || initialFailed || readyUploaded ? (
        <div className="grid gap-4">
          <div className="grid gap-1.5">
            <label htmlFor="media-editor-title" className="text-sm font-medium">{t('media.fields.title')}</label>
            <Input id="media-editor-title" value={title} onChange={(event) => setTitle(event.target.value)} disabled={!canEdit || !online || saveMutation.isPending} maxLength={200} />
          </div>
          {media.content?.type === 'video' ? (
            <p className="text-sm text-[var(--color-muted-foreground)]">
              {t('media.editor.metadataVideo', {
                duration: formatMediaDuration(media.content.duration_ms),
                width: media.content.width,
                height: media.content.height,
              })}
            </p>
          ) : null}
          {media.content?.type === 'audio' ? (
            <p className="text-sm text-[var(--color-muted-foreground)]">
              {t('media.editor.metadataAudio', { duration: formatMediaDuration(media.content.duration_ms) })}
            </p>
          ) : null}
          {previewUrl && media.content?.type === 'video' ? (
            <video className="w-full rounded-lg border border-[var(--color-border)]" controls src={previewUrl} />
          ) : null}
          {previewUrl && media.content?.type === 'audio' ? (
            <audio className="w-full" controls src={previewUrl} />
          ) : null}
          {canEdit ? (
            <div className="flex flex-wrap gap-2">
              <input ref={fileInputRef} type="file" className="hidden" accept={displayKind === 'video' ? 'video/*,audio/*' : 'audio/*,video/*'} onChange={(event) => {
                const file = event.target.files?.[0]
                if (file) uploadMutation.mutate(file)
                event.target.value = ''
              }} />
              {(initialFailed || readyUploaded) && !processingActive ? (
                <Button type="button" variant="outline" disabled={!online || uploadMutation.isPending} onClick={() => fileInputRef.current?.click()}>
                  {initialFailed ? t('media.upload.retry') : t('media.upload.replace')}
                </Button>
              ) : null}
              {media.pending_revision?.status === 'processing' ? (
                <Button type="button" variant="outline" disabled={cancelMutation.isPending} onClick={() => cancelMutation.mutate()}>
                  {t('media.upload.cancel')}
                </Button>
              ) : null}
            </div>
          ) : null}
          {uploadProgress != null ? (
            <div role="status" className="grid gap-1">
              <span className="text-sm">{t('media.upload.progress', { percent: Math.round(uploadProgress * 100) })}</span>
              <div className="h-2 rounded-full bg-[var(--color-muted)]">
                <div className="h-2 rounded-full bg-[var(--color-primary)]" style={{ width: `${Math.round(uploadProgress * 100)}%` }} />
              </div>
            </div>
          ) : null}
          {canEdit ? (
            <div className="flex justify-end">
              <Button disabled={!online || saveMutation.isPending} onClick={() => { setError(''); saveMutation.mutate() }}>
                {saveMutation.isPending ? t('common.load') : t('media.actions.save')}
              </Button>
            </div>
          ) : null}
        </div>
      ) : editableUrl ? (
        <>
          <div className="grid gap-1.5">
            <label htmlFor="media-editor-url" className="text-sm font-medium">{t(`media.fields.url.${kind}`)}</label>
            <Input id="media-editor-url" type="url" value={url} onChange={(event) => setUrl(event.target.value)} disabled={!canEdit || !online} />
          </div>
          {canEdit ? (
            <div className="flex justify-end">
              <Button disabled={!online || saveMutation.isPending} onClick={() => { setError(''); saveMutation.mutate() }}>
                {saveMutation.isPending ? t('common.load') : t('media.actions.save')}
              </Button>
            </div>
          ) : null}
        </>
      ) : (
        <p className="rounded-lg border border-[var(--color-border)] p-4 text-sm text-[var(--color-muted-foreground)]">{t('media.editor.notReady')}</p>
      )}

      {canonicalUrl ? (
        <div className="grid gap-1">
          <span className="text-sm font-medium">{t('media.fields.canonicalUrl')}</span>
          <a href={canonicalUrl} target="_blank" rel="noreferrer" className="break-all text-sm text-[var(--color-primary)] underline">{canonicalUrl}</a>
        </div>
      ) : null}

      {!canEdit ? <p role="status" className="text-sm text-[var(--color-muted-foreground)]">{t('media.editor.readOnly')}</p> : null}
      {error ? <p role="alert" className="text-sm text-[var(--color-destructive)]">{error}</p> : null}
    </section>
  )
}
