import { useEffect, useState } from 'react'

import { mediaAssetDataUrl } from '@/api/media-upload'

type PreviewKind = 'image' | 'pdf' | 'unknown'

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

export function DeckPagePreview({
  mediaId,
  blobId,
  label,
}: {
  mediaId: string
  blobId: string
  label: string
}) {
  const url = mediaAssetDataUrl(mediaId, blobId)
  const [objectUrl, setObjectUrl] = useState<string | null>(null)
  const [kind, setKind] = useState<PreviewKind>('unknown')
  const [error, setError] = useState(false)

  useEffect(() => {
    let revoked: string | null = null
    let cancelled = false
    void sniffPreview(url)
      .then(async ({ kind: nextKind, blob }) => {
        if (cancelled) return
        if (nextKind === 'pdf') {
          const { getDocument, GlobalWorkerOptions } = await import('pdfjs-dist')
          const worker = await import('pdfjs-dist/build/pdf.worker.min.mjs?url')
          GlobalWorkerOptions.workerSrc = worker.default
          const pdf = await getDocument({ data: await blob.arrayBuffer() }).promise
          const page = await pdf.getPage(1)
          const viewport = page.getViewport({ scale: 1.2 })
          const canvas = document.createElement('canvas')
          canvas.width = viewport.width
          canvas.height = viewport.height
          const context = canvas.getContext('2d')
          if (!context) throw new Error('preview_failed')
          await page.render({ canvas, canvasContext: context, viewport }).promise
          const nextUrl = canvas.toDataURL('image/png')
          if (!cancelled) {
            setKind('pdf')
            setObjectUrl(nextUrl)
          }
          return
        }
        const nextUrl = URL.createObjectURL(blob)
        revoked = nextUrl
        if (!cancelled) {
          setKind('image')
          setObjectUrl(nextUrl)
        }
      })
      .catch(() => {
        if (!cancelled) setError(true)
      })
    return () => {
      cancelled = true
      if (revoked) URL.revokeObjectURL(revoked)
    }
  }, [url])

  if (error) {
    return <div className="flex aspect-[4/3] items-center justify-center rounded-md border border-[var(--color-border)] bg-[var(--color-muted)]/30 text-xs text-[var(--color-muted-foreground)]">{label}</div>
  }
  if (!objectUrl) {
    return <div className="aspect-[4/3] animate-pulse rounded-md border border-[var(--color-border)] bg-[var(--color-muted)]/30" aria-hidden />
  }
  return (
    <img
      src={objectUrl}
      alt={label}
      className="aspect-[4/3] w-full rounded-md border border-[var(--color-border)] object-contain bg-[var(--color-muted)]/20"
      draggable={false}
      data-preview-kind={kind}
    />
  )
}
