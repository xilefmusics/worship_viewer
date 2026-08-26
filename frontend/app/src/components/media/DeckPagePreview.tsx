import { MediaDeckPageView } from '@/components/media/MediaDeckPageView'

export function DeckPagePreview({
  mediaId,
  blobId,
  label,
}: {
  mediaId: string
  blobId: string
  label: string
}) {
  return <MediaDeckPageView mediaId={mediaId} blobId={blobId} label={label} variant="thumb" />
}
