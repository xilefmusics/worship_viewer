import { useTranslation } from 'react-i18next'

import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { Team } from '@/api/teams-sessions-fetch'
import type { CreateMediaKind } from '@/lib/media-display'
import { isUploadMediaKind, isUrlMediaKind } from '@/lib/media-display'
import { getTeamDisplayName } from '@/lib/team-display-name'

export function MediaFields({
  title,
  kind,
  url,
  owner,
  teams,
  userId,
  disabled,
  showTeam,
  uploadFileName,
  onTitleChange,
  onKindChange,
  onUrlChange,
  onOwnerChange,
  onFileChange,
}: {
  title: string
  kind: CreateMediaKind
  url: string
  owner: string
  teams: Team[]
  userId?: string
  disabled?: boolean
  showTeam: boolean
  uploadFileName?: string
  onTitleChange: (value: string) => void
  onKindChange: (value: CreateMediaKind) => void
  onUrlChange: (value: string) => void
  onOwnerChange: (value: string) => void
  onFileChange?: (file: File | null) => void
}) {
  const { t } = useTranslation()
  const uploadKind = isUploadMediaKind(kind)
  return (
    <div className="grid gap-4">
      <div className="grid gap-1.5">
        <label htmlFor="media-title" className="text-sm font-medium">{t('media.fields.title')}</label>
        <Input id="media-title" value={title} onChange={(event) => onTitleChange(event.target.value)} disabled={disabled} maxLength={200} autoComplete="off" />
      </div>
      <div className="grid gap-1.5">
        <label htmlFor="media-kind" className="text-sm font-medium">{t('media.fields.kind')}</label>
        <Select value={kind} onValueChange={(value) => onKindChange(value as CreateMediaKind)} disabled={disabled}>
          <SelectTrigger id="media-kind"><SelectValue /></SelectTrigger>
          <SelectContent>
            <SelectItem value="youtube">{t('media.kinds.youtube')}</SelectItem>
            <SelectItem value="livestream">{t('media.kinds.livestream')}</SelectItem>
            <SelectItem value="web_page">{t('media.kinds.web_page')}</SelectItem>
            <SelectItem value="video">{t('media.kinds.video')}</SelectItem>
            <SelectItem value="audio">{t('media.kinds.audio')}</SelectItem>
          </SelectContent>
        </Select>
      </div>
      {uploadKind ? (
        <div className="grid gap-1.5">
          <label htmlFor="media-file" className="text-sm font-medium">{t(`media.fields.file.${kind}`)}</label>
          <Input
            id="media-file"
            type="file"
            accept={kind === 'video' ? 'video/*,audio/*' : 'audio/*,video/*'}
            disabled={disabled}
            onChange={(event) => onFileChange?.(event.target.files?.[0] ?? null)}
          />
          {uploadFileName ? <p className="text-sm text-[var(--color-muted-foreground)]">{uploadFileName}</p> : null}
        </div>
      ) : isUrlMediaKind(kind) ? (
        <div className="grid gap-1.5">
          <label htmlFor="media-url" className="text-sm font-medium">{t(`media.fields.url.${kind}`)}</label>
          <Input id="media-url" type="url" inputMode="url" value={url} onChange={(event) => onUrlChange(event.target.value)} placeholder="https://" disabled={disabled} autoCapitalize="none" autoCorrect="off" />
        </div>
      ) : null}
      {showTeam ? (
        <div className="grid gap-1.5">
          <label htmlFor="media-owner" className="text-sm font-medium">{t('media.fields.team')}</label>
          <Select value={owner} onValueChange={onOwnerChange} disabled={disabled}>
            <SelectTrigger id="media-owner"><SelectValue /></SelectTrigger>
            <SelectContent>
              {teams.map((team) => <SelectItem key={team.id} value={team.id}>{getTeamDisplayName(team, userId, t)}</SelectItem>)}
            </SelectContent>
          </Select>
        </div>
      ) : null}
    </div>
  )
}
