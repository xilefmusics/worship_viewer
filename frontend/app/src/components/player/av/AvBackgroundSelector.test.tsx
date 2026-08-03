import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'

import { AvBackgroundSelector } from '@/components/player/av/AvBackgroundSelector'
import { DEFAULT_AV_PREFERENCES } from '@/lib/player/av-preferences'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}))

vi.mock('@/components/player/av/AvSlideView', () => ({
  AvSlideView: ({ backgroundLayer }: { backgroundLayer: { preset: number } }) => (
    <div data-testid={`background-preview-${backgroundLayer.preset}`} />
  ),
}))

describe('AvBackgroundSelector', () => {
  it('shows and selects both Zeltlager background presets', async () => {
    const user = userEvent.setup()
    const onSelectPreset = vi.fn()

    render(
      <AvBackgroundSelector
        preset={2}
        previewText="Lyrics"
        contentLayer={DEFAULT_AV_PREFERENCES.contentLayer}
        onSelectPreset={onSelectPreset}
      />,
    )

    await user.click(
      screen.getByRole('button', { name: 'player.av.backgroundExpand' }),
    )

    const zeltlager1 = screen.getByRole('radio', {
      name: 'settings.playerRoles.background.zeltlager1',
    })
    const zeltlager2 = screen.getByRole('radio', {
      name: 'settings.playerRoles.background.zeltlager2',
    })

    expect(zeltlager1).toBeInTheDocument()
    expect(zeltlager2).toBeInTheDocument()
    expect(screen.getByTestId('background-preview-3')).toBeInTheDocument()
    expect(screen.getByTestId('background-preview-4')).toBeInTheDocument()

    await user.click(zeltlager2)
    expect(onSelectPreset).toHaveBeenCalledWith(4)
  })
})
