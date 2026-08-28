import { createFileRoute } from '@tanstack/react-router'

import { MediaEditorScreen } from '@/components/media/MediaEditorScreen'
import { parsePlayerEditorReturnSearch } from '@/lib/player/player-editor-return'

export const Route = createFileRoute('/_hub/media/$mediaId')({
  validateSearch: (search: Record<string, unknown>) => {
    const returnToPlayer = parsePlayerEditorReturnSearch(search)
    return {
      playerType: returnToPlayer?.playerType,
      playerId: returnToPlayer?.playerId,
      playerIndex: returnToPlayer?.playerIndex,
    }
  },
  component: MediaEditorRoute,
})

function MediaEditorRoute() {
  const { mediaId } = Route.useParams()
  return <MediaEditorScreen mediaId={mediaId} />
}
