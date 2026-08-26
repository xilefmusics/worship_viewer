import { useTranslation } from 'react-i18next'

import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { Team } from '@/api/teams-sessions-fetch'
import type { UrlMediaKind } from '@/lib/media-display'
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
  onTitleChange,
  onKindChange,
  onUrlChange,
  onOwnerChange,
}: {
  title: string
  kind: UrlMediaKind
  url: string
  owner: string
  teams: Team[]
  userId?: string
  disabled?: boolean
  showTeam: boolean
  onTitleChange: (value: string) => void
  onKindChange: (value: UrlMediaKind) => void
  onUrlChange: (value: string) => void
  onOwnerChange: (value: string) => void
}) {
  const { t } = useTranslation()
  return (
    <div className="grid gap-4">
      <div className="grid gap-1.5">
        <label htmlFor="media-title" className="text-sm font-medium">{t('media.fields.title')}</label>
        <Input id="media-title" value={title} onChange={(event) => onTitleChange(event.target.value)} disabled={disabled} maxLength={200} autoComplete="off" />
      </div>
      <div className="grid gap-1.5">
        <label htmlFor="media-kind" className="text-sm font-medium">{t('media.fields.kind')}</label>
        <Select value={kind} onValueChange={(value) => onKindChange(value as UrlMediaKind)} disabled={disabled}>
          <SelectTrigger id="media-kind"><SelectValue /></SelectTrigger>
          <SelectContent>
            <SelectItem value="youtube">{t('media.kinds.youtube')}</SelectItem>
            <SelectItem value="livestream">{t('media.kinds.livestream')}</SelectItem>
            <SelectItem value="web_page">{t('media.kinds.web_page')}</SelectItem>
          </SelectContent>
        </Select>
      </div>
      <div className="grid gap-1.5">
        <label htmlFor="media-url" className="text-sm font-medium">{t(`media.fields.url.${kind}`)}</label>
        <Input id="media-url" type="url" inputMode="url" value={url} onChange={(event) => onUrlChange(event.target.value)} placeholder="https://" disabled={disabled} autoCapitalize="none" autoCorrect="off" />
      </div>
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
