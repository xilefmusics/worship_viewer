import { createFileRoute } from '@tanstack/react-router'

import { MediaEditorScreen } from '@/components/media/MediaEditorScreen'

export const Route = createFileRoute('/_hub/media/$mediaId')({
  component: MediaEditorRoute,
})

function MediaEditorRoute() {
  const { mediaId } = Route.useParams()
  return <MediaEditorScreen mediaId={mediaId} />
}
