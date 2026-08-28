import { useEffect, useRef, useState } from 'react'

import { mediaAssetDataUrl } from '@/api/media-upload'
import { cn } from '@/lib/utils'

type PreviewKind = 'image' | 'pdf' | 'unknown'

export type MediaDeckPageStatus = 'loading' | 'ready' | 'error'

async function sniffPreview(url: string): Promise<{ kind: PreviewKind; blob: Blob }> {
  const response = await fetch(url, { credentials: 'include' })
  if (!response.ok) throw new Error('preview_failed')
  const blob = await response.blob()
  const header = new Uint8Array(await blob.slice(0, 5).arrayBuffer())
  if (header[0] === 0x89 && header[1] === 0x50) return { kind: 'image', blob }
  if (header[0] === 0xff && header[1] === 0xd8) return { kind: 'image', blob }
  if (header[0] === 0x25 && header[1] === 0x50 && header[2] === 0x44 && header[3] === 0x46) {
    return { kind: 'pdf', blob }
  }
  const type = blob.type.toLowerCase()
  if (type.includes('svg') || type.includes('png') || type.includes('jpeg')) return { kind: 'image', blob }
  if (type.includes('pdf')) return { kind: 'pdf', blob }
  return { kind: 'image', blob }
}

export function MediaDeckPageView({
  mediaId,
  blobId,
  label,
  variant = 'contain',
  className,
  onStatus,
}: {
  mediaId: string
  blobId: string
  label: string
  variant?: 'thumb' | 'contain'
  className?: string
  onStatus?: (status: Exclude<MediaDeckPageStatus, 'loading'>) => void
}) {
  const url = mediaAssetDataUrl(mediaId, blobId)
  const [result, setResult] = useState<{
    url: string
    objectUrl: string | null
    kind: PreviewKind
    error: boolean
  } | null>(null)
  const onStatusRef = useRef(onStatus)

  useEffect(() => {
    onStatusRef.current = onStatus
  }, [onStatus])

  useEffect(() => {
    let cancelled = false
    let cleanupPdf: (() => void) | null = null
    void sniffPreview(url)
      .then(async ({ kind: nextKind, blob }) => {
        if (cancelled) return
        if (nextKind === 'pdf') {
          const { getDocument, GlobalWorkerOptions } = await import('pdfjs-dist')
          const worker = await import('pdfjs-dist/build/pdf.worker.min.mjs?url')
          GlobalWorkerOptions.workerSrc = worker.default
          const pdf = await getDocument({ data: await blob.arrayBuffer() }).promise
          cleanupPdf = () => {
            const destroy = (pdf as { destroy?: () => Promise<unknown> }).destroy
            void destroy?.()
          }
          const page = await pdf.getPage(1)
          const viewport = page.getViewport({ scale: variant === 'thumb' ? 1.2 : 2 })
          const canvas = document.createElement('canvas')
          canvas.width = viewport.width
          canvas.height = viewport.height
          const context = canvas.getContext('2d')
          if (!context) throw new Error('preview_failed')
          await page.render({ canvas, canvasContext: context, viewport }).promise
          const nextUrl = canvas.toDataURL('image/png')
          if (!cancelled) {
            setResult({ url, objectUrl: nextUrl, kind: 'pdf', error: false })
            onStatusRef.current?.('ready')
          }
          return
        }
        const nextUrl = URL.createObjectURL(blob)
        if (cancelled) {
          URL.revokeObjectURL(nextUrl)
          return
        }
        setResult({ url, objectUrl: nextUrl, kind: 'image', error: false })
        onStatusRef.current?.('ready')
      })
      .catch(() => {
        if (!cancelled) {
          setResult({ url, objectUrl: null, kind: 'unknown', error: true })
          onStatusRef.current?.('error')
        }
      })
    return () => {
      cancelled = true
      cleanupPdf?.()
    }
  }, [url, variant])

  useEffect(() => {
    const objectUrl = result?.kind === 'image' ? result.objectUrl : null
    return () => {
      if (objectUrl) URL.revokeObjectURL(objectUrl)
    }
  }, [result])

  const thumb = variant === 'thumb'
  // Keep the last rendered frame visible while the next asset is fetched and decoded.
  const active = result
  const error = active?.error ?? false
  const objectUrl = active?.objectUrl ?? null
  const kind = active?.kind ?? 'unknown'

  if (error) {
    return (
      <div
        className={cn(
          'flex items-center justify-center bg-[var(--color-muted)]/30 text-xs text-[var(--color-muted-foreground)]',
          thumb && 'aspect-[4/3] rounded-md border border-[var(--color-border)]',
          !thumb && 'h-full w-full',
          className,
        )}
      >
        {label}
      </div>
    )
  }
  if (!objectUrl) {
    return (
      <div
        className={cn(
          'animate-pulse bg-[var(--color-muted)]/30',
          thumb && 'aspect-[4/3] rounded-md border border-[var(--color-border)]',
          !thumb && 'h-full w-full',
          className,
        )}
        aria-hidden
      />
    )
  }
  return (
    <img
      src={objectUrl}
      alt={label}
      className={cn(
        'min-h-0 min-w-0 max-h-full max-w-full object-contain',
        thumb &&
          'media-deck-page-view--thumb aspect-[4/3] h-full w-full rounded-md border border-[var(--color-border)] bg-[var(--color-muted)]/20',
        !thumb && 'media-deck-page-view--contain av-slide-view__deck-image',
        className,
      )}
      draggable={false}
      data-preview-kind={kind}
    />
  )
}
