import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { AvSlideView } from '@/components/player/av/AvSlideView'
import { DEFAULT_AV_PREFERENCES } from '@/lib/player/av-preferences'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}))

vi.mock('motion/react', () => ({
  useReducedMotion: () => false,
}))

describe('AvSlideView', () => {
  it('adds opposite-contrast treatment only when the background is omitted', () => {
    const contentLayer = {
      ...DEFAULT_AV_PREFERENCES.contentLayer,
      primaryTextLightness: 0,
      secondaryTextLightness: 100,
      textShadow: 'none' as const,
    }
    const view = (showBackground: boolean) => (
      <AvSlideView
        contentLines={[{ primary: 'Black lyrics', secondary: 'White translation' }]}
        contentLayer={contentLayer}
        backgroundLayer={DEFAULT_AV_PREFERENCES.backgroundLayer}
        transition={DEFAULT_AV_PREFERENCES.transition}
        screenState="live"
        compact
        showBackground={showBackground}
      />
    )
    const { container, rerender } = render(view(false))

    expect(container.firstElementChild).toHaveClass(
      'av-slide-view--transparent-preview',
    )
    expect(screen.getByText('Black lyrics')).toHaveClass(
      'av-slide-content__line--shadow-white',
    )
    expect(screen.getByText('White translation')).toHaveClass(
      'av-slide-content__line--shadow-black',
    )

    rerender(view(true))

    expect(container.firstElementChild).not.toHaveClass(
      'av-slide-view--transparent-preview',
    )
  })
})
