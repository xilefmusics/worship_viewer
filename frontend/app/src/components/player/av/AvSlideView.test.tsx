import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { AvSlideView } from '@/components/player/av/AvSlideView'
import { DEFAULT_AV_PREFERENCES } from '@/lib/player/av-preferences'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}))

vi.mock('motion/react', () => ({
  useReducedMotion: () => false,
  AnimatePresence: ({ children }: { children?: unknown }) => children,
  motion: {
    div: ({ children, className }: { children?: unknown; className?: string }) => (
      <div className={className}>{children as never}</div>
    ),
  },
}))

vi.mock('@/components/player/av/AvSlideScaledStage', () => ({
  AvSlideScaledStage: ({ children }: { children?: unknown }) => <div>{children as never}</div>,
}))

vi.mock('@/components/media/MediaDeckPageView', () => ({
  MediaDeckPageView: ({ mediaId, blobId }: { mediaId: string; blobId: string }) => (
    <img alt="deck-page" data-testid="deck-page" data-media={mediaId} data-asset={blobId} />
  ),
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

  it('contains a live deck page and hides it for blank and blackout', () => {
    const view = (screenState: 'live' | 'blank' | 'blackout') => (
      <AvSlideView
        deckPage={{ mediaId: 'media-1', assetId: 'page-a' }}
        contentLayer={DEFAULT_AV_PREFERENCES.contentLayer}
        backgroundLayer={DEFAULT_AV_PREFERENCES.backgroundLayer}
        transition={DEFAULT_AV_PREFERENCES.transition}
        screenState={screenState}
      />
    )
    const { container, rerender } = render(view('live'))
    expect(screen.getByTestId('deck-page')).toHaveAttribute('data-asset', 'page-a')
    expect(container.firstElementChild).toHaveClass('av-slide-view--deck')
    expect(container.querySelector('.av-slide-view__background')).not.toBeInTheDocument()
    expect(container.querySelector('.av-slide-view__animated-layer')).not.toBeInTheDocument()

    rerender(view('blank'))
    expect(screen.queryByTestId('deck-page')).not.toBeInTheDocument()
    expect(screen.getByText('player.av.blankOn')).toBeInTheDocument()

    rerender(view('blackout'))
    expect(screen.queryByTestId('deck-page')).not.toBeInTheDocument()
    expect(document.querySelector('.av-slide-view--blackout')).toBeTruthy()
  })
})
