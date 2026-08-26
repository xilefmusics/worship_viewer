import * as Dialog from '@radix-ui/react-dialog'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { uploadMediaSource } from '@/api/media-upload'
import { createMedia, mediaListRootKey } from '@/api/media'
import { MediaFields } from '@/components/media/MediaFields'
import { Button } from '@/components/ui/button'
import { useWritableTeams } from '@/hooks/useWritableTeams'
import {
  isCreateMediaKind,
  isUploadMediaKind,
  isValidUrlMediaInput,
  type CreateMediaKind,
  uploadCreateContent,
  urlContent,
} from '@/lib/media-display'

export function CreateMediaDialog({ open, onOpenChange, onCreated }: { open: boolean; onOpenChange: (open: boolean) => void; onCreated: (id: string) => void }) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const { teams, user, isPending: teamsPending } = useWritableTeams('mediaCreate', open)
  const [title, setTitle] = useState('')
  const [kind, setKind] = useState<CreateMediaKind>('youtube')
  const [url, setUrl] = useState('')
  const [owner, setOwner] = useState('')
  const [file, setFile] = useState<File | null>(null)
  const [uploadProgress, setUploadProgress] = useState<number | null>(null)
  const [error, setError] = useState('')

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- initialize the owner after the async team list arrives
    if (open && !owner && teams[0]) setOwner(teams[0].id)
  }, [open, owner, teams])

  const mutation = useMutation({
    mutationFn: async () => {
      if (!title.trim()) throw new Error(t('media.validation.titleRequired'))
      if (!owner) throw new Error(t('media.validation.noWritableTeam'))
      if (isUploadMediaKind(kind)) {
        if (!file) throw new Error(t('media.validation.fileRequired'))
        const created = await createMedia(queryClient, {
          title: title.trim(),
          owner,
          content: uploadCreateContent(kind),
        })
        setUploadProgress(0)
        await uploadMediaSource({
          mediaId: created.id,
          kind,
          file,
          onProgress: (ratio) => setUploadProgress(ratio),
        })
        return created
      }
      if (!isValidUrlMediaInput(kind, url)) throw new Error(t('media.validation.invalidUrl'))
      return createMedia(queryClient, {
        title: title.trim(),
        owner,
        content: urlContent(kind, url.trim()),
      })
    },
    onSuccess: (created) => {
      void queryClient.invalidateQueries({ queryKey: mediaListRootKey })
      onCreated(created.id)
    },
    onError: (cause: Error) => {
      setUploadProgress(null)
      const key = cause.message
      if (key === 'payload_too_large') setError(t('media.upload.errors.tooLarge'))
      else if (key === 'upload_failed') setError(t('media.upload.errors.failed'))
      else setError(cause.message)
    },
  })

  function close(next: boolean) {
    onOpenChange(next)
    if (!next) {
      setTitle('')
      setKind('youtube')
      setUrl('')
      setOwner('')
      setFile(null)
      setUploadProgress(null)
      setError('')
    }
  }

  const busy = mutation.isPending
  const uploading = uploadProgress != null

  return (
    <Dialog.Root open={open} onOpenChange={close}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-black/40" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 grid max-h-[90dvh] w-[min(32rem,calc(100%-2rem))] -translate-x-1/2 -translate-y-1/2 gap-5 overflow-y-auto rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-[var(--shadow-elevated)]">
          <div className="grid gap-1.5">
            <Dialog.Title className="text-lg font-semibold">{t('media.create.title')}</Dialog.Title>
            <Dialog.Description className="text-sm text-[var(--color-muted-foreground)]">{t('media.create.description')}</Dialog.Description>
          </div>
          <MediaFields
            title={title}
            kind={kind}
            url={url}
            owner={owner}
            teams={teams}
            userId={user?.id}
            showTeam={teams.length > 1}
            disabled={busy}
            uploadFileName={file?.name}
            onTitleChange={setTitle}
            onKindChange={(value) => {
              if (isCreateMediaKind(value)) setKind(value)
            }}
            onUrlChange={setUrl}
            onOwnerChange={setOwner}
            onFileChange={setFile}
          />
          {uploading ? (
            <div role="status" className="grid gap-1">
              <span className="text-sm">{t('media.upload.progress', { percent: Math.round(uploadProgress * 100) })}</span>
              <div className="h-2 rounded-full bg-[var(--color-muted)]">
                <div className="h-2 rounded-full bg-[var(--color-primary)] transition-[width]" style={{ width: `${Math.round(uploadProgress * 100)}%` }} />
              </div>
            </div>
          ) : null}
          {error ? <p role="alert" className="text-sm text-[var(--color-destructive)]">{error}</p> : null}
          {!teamsPending && teams.length === 0 ? <p role="status" className="text-sm text-[var(--color-muted-foreground)]">{t('media.validation.noWritableTeam')}</p> : null}
          <div className="flex justify-end gap-2">
            <Button type="button" variant="outline" onClick={() => close(false)} disabled={busy}>{t('common.cancel')}</Button>
            <Button type="button" disabled={busy || teamsPending || teams.length === 0} onClick={() => { setError(''); mutation.mutate() }}>{busy ? t('common.load') : t('media.actions.create')}</Button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
