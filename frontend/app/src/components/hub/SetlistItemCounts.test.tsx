import { render, screen } from '@testing-library/react'
import i18next from 'i18next'
import { I18nextProvider, initReactI18next } from 'react-i18next'
import { describe, expect, it } from 'vitest'

import { SetlistItemCounts } from '@/components/hub/SetlistItemCounts'
import de from '@/i18n/de.json'
import en from '@/i18n/en.json'
import type { SetlistItem } from '@/lib/setlist-items'

const song = (id: string): SetlistItem => ({
  type: 'song',
  id,
  nr: null,
  key: null,
  tempo: null,
  language: null,
  flow: null,
})

async function renderCounts(language: 'en' | 'de', items: SetlistItem[]) {
  const i18n = i18next.createInstance()
  await i18n.use(initReactI18next).init({
    resources: { en: { translation: en }, de: { translation: de } },
    lng: language,
    fallbackLng: 'en',
    interpolation: { escapeValue: false },
  })

  return render(
    <I18nextProvider i18n={i18n}>
      <p data-testid="counts"><SetlistItemCounts items={items} /></p>
    </I18nextProvider>,
  )
}

describe('SetlistItemCounts', () => {
  it.each([
    ['en' as const, '1 song · 1 media item', '2 songs · 2 media items'],
    ['de' as const, '1 Lied · 1 Medium', '2 Lieder · 2 Medien'],
  ])('renders singular and plural counts in %s', async (language, singular, plural) => {
    const { unmount } = await renderCounts(language, [song('s1'), { type: 'media', id: 'm1' }])
    expect(screen.getByTestId('counts')).toHaveTextContent(singular)
    unmount()

    await renderCounts(language, [song('s1'), song('s2'), { type: 'media', id: 'm1' }, { type: 'media', id: 'm2' }])
    expect(screen.getByTestId('counts')).toHaveTextContent(plural)
  })

  it.each([
    ['en' as const, '0 songs', '2 songs'],
    ['de' as const, '0 Lieder', '2 Lieder'],
  ])('omits the zero-media segment in %s', async (language, empty, songsOnly) => {
    const { unmount } = await renderCounts(language, [])
    expect(screen.getByTestId('counts')).toHaveTextContent(empty)
    expect(screen.getByTestId('counts')).not.toHaveTextContent('·')
    unmount()

    await renderCounts(language, [song('s1'), song('s2')])
    expect(screen.getByTestId('counts')).toHaveTextContent(songsOnly)
    expect(screen.getByTestId('counts')).not.toHaveTextContent('·')
  })
})
