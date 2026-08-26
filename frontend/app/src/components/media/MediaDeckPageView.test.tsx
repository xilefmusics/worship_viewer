import { render, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { MediaDeckPageView } from '@/components/media/MediaDeckPageView'

describe('MediaDeckPageView', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('renders an image asset with contain sizing and reports ready', async () => {
    const onStatus = vi.fn()
    const bytes = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d])
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        blob: () => Promise.resolve(new Blob([bytes], { type: 'image/png' })),
      }),
    )
    vi.stubGlobal('URL', {
      ...URL,
      createObjectURL: () => 'blob:page',
      revokeObjectURL: vi.fn(),
    })

    const { container } = render(
      <MediaDeckPageView mediaId="m1" blobId="a1" label="Page 1" variant="contain" onStatus={onStatus} />,
    )

    await waitFor(() => {
      expect(container.querySelector('img')?.getAttribute('src')).toBe('blob:page')
    })
    expect(container.querySelector('img')).toHaveClass('av-slide-view__deck-image')
    expect(onStatus).toHaveBeenCalledWith('ready')
  })

  it('reports a safe error when the asset cannot be loaded', async () => {
    const onStatus = vi.fn()
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({ ok: false }),
    )

    const { container } = render(
      <MediaDeckPageView mediaId="m1" blobId="a1" label="Page 1" onStatus={onStatus} />,
    )

    await waitFor(() => {
      expect(onStatus).toHaveBeenCalledWith('error')
    })
    expect(container.textContent).toContain('Page 1')
  })
})
