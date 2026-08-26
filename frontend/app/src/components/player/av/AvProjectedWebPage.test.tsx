import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { AvProjectedWebPage } from '@/components/player/av/AvProjectedWebPage'
import { DEFAULT_AV_PREFERENCES } from '@/lib/player/av-preferences'
import { buildAvPlaybackIntent } from '@/lib/player/av-projection-playback'
import { buildAvProjectionCommand } from '@/lib/player/av-projection-protocol'
import {
  WEB_PAGE_FORBIDDEN_SANDBOX,
  WEB_PAGE_IFRAME_SANDBOX,
  webPageIframeSandboxTokens,
} from '@/lib/player/av-web-page-embed'

const layers = {
  contentLayer: DEFAULT_AV_PREFERENCES.contentLayer,
  backgroundLayer: DEFAULT_AV_PREFERENCES.backgroundLayer,
  transition: DEFAULT_AV_PREFERENCES.transition,
}

function webCommand(
  commandId: number,
  playback = buildAvPlaybackIntent({ action: 'play' }),
  url = 'https://example.com/bulletin',
) {
  return buildAvProjectionCommand({
    sessionId: 'shared',
    commandId,
    ...layers,
    screenState: 'live',
    itemTitle: 'Bulletin',
    nextPreview: null,
    content: { type: 'web_page', url },
    playback,
  })
}

describe('AvProjectedWebPage', () => {
  it('I5: shows a least-privileged sandboxed iframe on Show', () => {
    render(<AvProjectedWebPage command={webCommand(1)} onAck={vi.fn()} />)
    const iframe = screen.getByTestId('av-projected-web-iframe')
    expect(iframe).toHaveAttribute('src', 'https://example.com/bulletin')
    expect(iframe).toHaveAttribute('sandbox', WEB_PAGE_IFRAME_SANDBOX)
    expect(iframe).toHaveAttribute('referrerpolicy', 'no-referrer')
    const tokens = webPageIframeSandboxTokens(iframe.getAttribute('sandbox'))
    expect(tokens).toEqual(['allow-scripts'])
    for (const forbidden of WEB_PAGE_FORBIDDEN_SANDBOX) {
      expect(tokens).not.toContain(forbidden)
    }
  })

  it('I5: Hide unmounts the iframe and Reload remounts the same URL', () => {
    const onAck = vi.fn()
    const { rerender } = render(<AvProjectedWebPage command={webCommand(1)} onAck={onAck} />)
    expect(screen.getByTestId('av-projected-web-iframe')).toBeInTheDocument()
    rerender(
      <AvProjectedWebPage
        command={webCommand(2, buildAvPlaybackIntent({ action: 'pause' }))}
        onAck={onAck}
      />,
    )
    expect(screen.queryByTestId('av-projected-web-iframe')).not.toBeInTheDocument()
    expect(screen.getByTestId('av-projected-web')).toHaveClass('av-projected-media--hidden')
    rerender(
      <AvProjectedWebPage
        command={webCommand(3, buildAvPlaybackIntent({ action: 'restart' }))}
        onAck={onAck}
      />,
    )
    expect(screen.getByTestId('av-projected-web-iframe')).toHaveAttribute(
      'src',
      'https://example.com/bulletin',
    )
  })

  it('I5: rejects unsafe URLs and reports embed failure from iframe error', () => {
    const onAck = vi.fn()
    const { rerender } = render(
      <AvProjectedWebPage command={webCommand(1, buildAvPlaybackIntent({ action: 'play' }), 'javascript:alert(1)')} onAck={onAck} />,
    )
    expect(screen.queryByTestId('av-projected-web-iframe')).not.toBeInTheDocument()
    expect(onAck).toHaveBeenCalledWith(false, expect.anything(), expect.objectContaining({ code: 'load_failed' }))

    onAck.mockClear()
    rerender(<AvProjectedWebPage command={webCommand(2)} onAck={onAck} />)
    const iframe = screen.getByTestId('av-projected-web-iframe')
    iframe.dispatchEvent(new Event('error'))
    expect(onAck).toHaveBeenCalledWith(
      false,
      expect.anything(),
      expect.objectContaining({ code: 'embed_blocked' }),
    )
  })
})
