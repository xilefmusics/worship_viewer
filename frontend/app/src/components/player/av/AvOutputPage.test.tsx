import { act, render, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { AvOutputPage } from '@/components/player/av/AvOutputPage'
import { DEFAULT_AV_PREFERENCES } from '@/lib/player/av-preferences'
import { buildAvProjectionCommand } from '@/lib/player/av-projection-protocol'

let slideViewProps: Record<string, unknown> | null = null
let onMessage: ((message: unknown) => void) | null = null
const send = vi.fn()

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}))

vi.mock('@/components/player/av/AvSlideView', () => ({
  AvSlideView: (props: Record<string, unknown>) => {
    slideViewProps = props
    return <div data-testid="slide-view" />
  },
}))

vi.mock('@/lib/player/av-projection-sync', () => ({
  createAvProjectionChannel: (_sessionId: string, listener: (message: unknown) => void) => {
    onMessage = listener
    return {
      send,
      readLatestCommand: () => null,
      close: vi.fn(),
    }
  },
  readOrCreateAvOutputId: () => 'out-1',
}))

const layers = {
  contentLayer: DEFAULT_AV_PREFERENCES.contentLayer,
  backgroundLayer: DEFAULT_AV_PREFERENCES.backgroundLayer,
  transition: DEFAULT_AV_PREFERENCES.transition,
}

beforeEach(() => {
  slideViewProps = null
  onMessage = null
  send.mockReset()
})

describe('AvOutputPage', () => {
  it('renders structured content lines when the command includes them', async () => {
    render(<AvOutputPage sessionId="shared" />)
    act(() => {
      onMessage?.(
        buildAvProjectionCommand({
          sessionId: 'shared',
          commandId: 1,
          ...layers,
          screenState: 'live',
          itemTitle: 'Song',
          nextPreview: null,
          content: {
            type: 'lyrics',
            contentText: 'Hello',
            contentLines: [{ primary: 'Hello', secondary: 'Hallo' }],
          },
        }),
      )
    })

    await waitFor(() => {
      expect(slideViewProps?.contentLines).toEqual([{ primary: 'Hello', secondary: 'Hallo' }])
    })
    expect(slideViewProps?.contentText).toBeUndefined()
    expect(slideViewProps?.screenState).toBe('live')
  })

  it('falls back to contentText when structured lines are absent', async () => {
    render(<AvOutputPage sessionId="shared" />)
    act(() => {
      onMessage?.(
        buildAvProjectionCommand({
          sessionId: 'shared',
          commandId: 1,
          ...layers,
          screenState: 'live',
          itemTitle: 'Song',
          nextPreview: null,
          content: { type: 'lyrics', contentText: 'Hello' },
        }),
      )
    })

    await waitFor(() => {
      expect(slideViewProps?.contentText).toBe('Hello')
    })
    expect(slideViewProps?.contentLines).toBeUndefined()
  })

  it('passes blank screen state without structured lines leaking through the view props', async () => {
    render(<AvOutputPage sessionId="shared" />)
    act(() => {
      onMessage?.(
        buildAvProjectionCommand({
          sessionId: 'shared',
          commandId: 2,
          ...layers,
          screenState: 'blank',
          itemTitle: 'Song',
          nextPreview: null,
          content: { type: 'lyrics', contentText: '' },
        }),
      )
    })

    await waitFor(() => {
      expect(slideViewProps?.screenState).toBe('blank')
    })
    expect(slideViewProps?.contentText).toBe('')
    expect(slideViewProps?.contentLines).toBeUndefined()
  })

  it('projects a deck page over the current background', async () => {
    render(<AvOutputPage sessionId="shared" />)
    act(() => {
      onMessage?.(
        buildAvProjectionCommand({
          sessionId: 'shared',
          commandId: 3,
          ...layers,
          screenState: 'live',
          itemTitle: 'Deck',
          nextPreview: null,
          content: { type: 'deck_page', mediaId: 'media-1', assetId: 'page-a' },
        }),
      )
    })

    await waitFor(() => {
      expect(slideViewProps?.deckPage).toEqual({ mediaId: 'media-1', assetId: 'page-a' })
    })
    expect(slideViewProps?.screenState).toBe('live')
  })
})
