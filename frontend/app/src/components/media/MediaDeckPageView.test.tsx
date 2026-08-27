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
    expect(container.querySelector('img')).toHaveClass(
      'media-deck-page-view--contain',
      'av-slide-view__deck-image',
      'max-h-full',
      'max-w-full',
    )
    expect(onStatus).toHaveBeenCalledWith('ready')
  })

  it('marks thumbnails as size-contained independently from full slide views', async () => {
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
      createObjectURL: () => 'blob:large-page',
      revokeObjectURL: vi.fn(),
    })

    const { container } = render(
      <div style={{ width: 288, height: 44, overflow: 'hidden' }}>
        <MediaDeckPageView mediaId="m1" blobId="large" label="Large page" variant="thumb" />
      </div>,
    )

    await waitFor(() => {
      expect(container.querySelector('img')?.getAttribute('src')).toBe('blob:large-page')
    })
    expect(container.querySelector('img')).toHaveClass(
      'media-deck-page-view--thumb',
      'aspect-[4/3]',
      'h-full',
      'w-full',
      'min-h-0',
      'min-w-0',
      'max-h-full',
      'max-w-full',
      'object-contain',
    )
    expect(container.querySelector('img')).not.toHaveClass('av-slide-view__deck-image')
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

  it('keeps the current frame visible until the next image is ready', async () => {
    const bytes = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d])
    let resolveSecond: ((value: { ok: boolean; blob: () => Promise<Blob> }) => void) | undefined
    const fetchMock = vi.fn()
      .mockResolvedValueOnce({
        ok: true,
        blob: () => Promise.resolve(new Blob([bytes], { type: 'image/png' })),
      })
      .mockReturnValueOnce(new Promise((resolve) => {
        resolveSecond = resolve
      }))
    vi.stubGlobal('fetch', fetchMock)
    const revokeObjectURL = vi.fn()
    let objectUrlIndex = 0
    vi.stubGlobal('URL', {
      ...URL,
      createObjectURL: () => `blob:page-${++objectUrlIndex}`,
      revokeObjectURL,
    })

    const { container, rerender } = render(
      <MediaDeckPageView mediaId="m1" blobId="a1" label="Page 1" />,
    )
    await waitFor(() => {
      expect(container.querySelector('img')?.getAttribute('src')).toBe('blob:page-1')
    })

    rerender(<MediaDeckPageView mediaId="m1" blobId="a2" label="Page 2" />)
    expect(container.querySelector('img')?.getAttribute('src')).toBe('blob:page-1')
    resolveSecond?.({
      ok: true,
      blob: () => Promise.resolve(new Blob([bytes], { type: 'image/png' })),
    })

    await waitFor(() => {
      expect(container.querySelector('img')?.getAttribute('src')).toBe('blob:page-2')
    })
    await waitFor(() => {
      expect(revokeObjectURL).toHaveBeenCalledWith('blob:page-1')
    })
  })
})
