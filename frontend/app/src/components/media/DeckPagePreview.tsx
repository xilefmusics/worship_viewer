import { MediaDeckPageView } from '@/components/media/MediaDeckPageView'

export function DeckPagePreview({
  mediaId,
  blobId,
  label,
  className,
}: {
  mediaId: string
  blobId: string
  label: string
  className?: string
}) {
  return (
    <MediaDeckPageView
      mediaId={mediaId}
      blobId={blobId}
      label={label}
      variant="thumb"
      className={`!aspect-video !h-auto !border-0 !rounded-none shadow-none ${className ?? ''}`}
    />
  )
}
