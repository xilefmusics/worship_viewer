import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

import { fetchMedia, mediaDetailKey, mediaListRootKey, updateMedia } from '@/api/media'
import { MediaFields } from '@/components/media/MediaFields'
import { Button } from '@/components/ui/button'
import { useOnline } from '@/hooks/use-online'
import { useTeamDetail } from '@/hooks/useTeamDetail'
import { useWritableTeams } from '@/hooks/useWritableTeams'
import {
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
  const detailQuery = useQuery({
    queryKey: mediaDetailKey(mediaId),
    queryFn: ({ signal }) => fetchMedia(queryClient, mediaId, signal),
  })
  const media = detailQuery.data
  const ownerTeam = useTeamDetail(media?.owner ?? '', { enabled: Boolean(media?.owner) })
  const { teams, user } = useWritableTeams(`mediaEditor:${mediaId}`, Boolean(media))
  const canEdit = Boolean(media && ownerTeam.data && user?.id && canEditTeamLibrary(ownerTeam.data, user.id))
  const [title, setTitle] = useState('')
  const [kind, setKind] = useState<UrlMediaKind>('youtube')
  const [url, setUrl] = useState('')
  const [owner, setOwner] = useState('')
  const [error, setError] = useState('')

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

  const teamChoices = useMemo(() => {
    const all = [...teams]
    if (ownerTeam.data && !all.some((team) => team.id === ownerTeam.data?.id)) all.unshift(ownerTeam.data)
    return all
  }, [ownerTeam.data, teams])
  const displayStatus = media ? mediaDisplayStatus(media) : 'unknown'
  const editableUrl = media
    ? displayStatus === 'ready' && isUrlMediaKind(mediaDisplayKind(media))
    : false

  const mutation = useMutation({
    mutationFn: async () => {
      if (!canEdit || !editableUrl) throw new Error(t('media.validation.notEditable'))
      if (!title.trim()) throw new Error(t('media.validation.titleRequired'))
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

  if (detailQuery.isPending) {
    return <div className="mx-auto w-full max-w-2xl py-12 text-center text-sm text-[var(--color-muted-foreground)]" role="status">{t('common.load')}</div>
  }
  if (detailQuery.error || !media) {
    return <div className="mx-auto flex w-full max-w-2xl flex-col items-center gap-3 py-12 text-center"><p className="text-sm text-[var(--color-muted-foreground)]">{t('media.editor.loadFailed')}</p><Button variant="outline" onClick={() => void detailQuery.refetch()}>{t('hub.error.retry')}</Button></div>
  }

  const status = displayStatus
  const canonicalUrl = mediaCanonicalUrl(media)
  return (
    <section className="mx-auto grid w-full max-w-2xl gap-6 pb-8" aria-labelledby="media-editor-heading">
      <div className="grid gap-1">
        <h1 id="media-editor-heading" className="text-xl font-semibold">{media.title}</h1>
        <p className="text-sm text-[var(--color-muted-foreground)]">{t(`media.states.${status}`)} · {t(`media.kinds.${mediaDisplayKind(media)}`)}</p>
      </div>
      {status === 'failed' ? <div role="alert" className="rounded-lg border border-[var(--color-destructive)]/40 bg-[var(--color-destructive)]/5 p-3 text-sm"><p className="font-medium">{t('media.editor.failedTitle')}</p><p className="text-[var(--color-muted-foreground)]">{media.pending_revision?.processing_error?.detail ?? t('media.editor.failedBody')}</p></div> : null}
      {status === 'processing' ? <div role="status" className="rounded-lg border border-[var(--color-border)] bg-[var(--color-muted)]/30 p-3 text-sm">{t('media.editor.processingBody')}</div> : null}
      {editableUrl ? (
        <MediaFields title={title} kind={kind} url={url} owner={owner} teams={teamChoices} userId={user?.id} showTeam={canEdit && teamChoices.length > 1} disabled={!canEdit || !online || mutation.isPending} onTitleChange={setTitle} onKindChange={setKind} onUrlChange={setUrl} onOwnerChange={setOwner} />
      ) : (
        <p className="rounded-lg border border-[var(--color-border)] p-4 text-sm text-[var(--color-muted-foreground)]">{t(status === 'ready' ? 'media.editor.futureKind' : 'media.editor.notReady')}</p>
      )}
      {canonicalUrl ? <div className="grid gap-1"><span className="text-sm font-medium">{t('media.fields.canonicalUrl')}</span><a href={canonicalUrl} target="_blank" rel="noreferrer" className="break-all text-sm text-[var(--color-primary)] underline">{canonicalUrl}</a></div> : null}
      {!canEdit ? <p role="status" className="text-sm text-[var(--color-muted-foreground)]">{t('media.editor.readOnly')}</p> : null}
      {error ? <p role="alert" className="text-sm text-[var(--color-destructive)]">{error}</p> : null}
      {canEdit && editableUrl ? <div className="flex justify-end"><Button disabled={!online || mutation.isPending} onClick={() => { setError(''); mutation.mutate() }}>{mutation.isPending ? t('common.load') : t('media.actions.save')}</Button></div> : null}
    </section>
  )
}
