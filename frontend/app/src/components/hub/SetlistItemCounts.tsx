import { useTranslation } from 'react-i18next'

import type { SetlistItem } from '@/lib/setlist-items'
import { countSetlistItems } from '@/lib/setlist-items'

export function SetlistItemCounts({ items }: { items: SetlistItem[] | null | undefined }) {
  const { t } = useTranslation()
  const counts = countSetlistItems(items)

  return (
    <>
      {t('hub.meta.songsCount', { count: counts.songs })}
      {counts.media > 0 ? ` · ${t('hub.meta.mediaCount', { count: counts.media })}` : null}
    </>
  )
}
