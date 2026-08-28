import { createFileRoute, Outlet, useLocation, useNavigate, useRouterState } from '@tanstack/react-router'
import { useEffect, useState } from 'react'

import { CreateMediaDialog } from '@/components/media/CreateMediaDialog'
import { MediaListView } from '@/components/media/MediaListView'
import { emptyEditorReturnSearch } from '@/lib/player/player-editor-return'

export const Route = createFileRoute('/_hub/media')({
  component: MediaRoute,
})

function MediaRoute() {
  const location = useLocation()
  const navigate = useNavigate()
  const pathname = useRouterState({ select: (state) => state.location.pathname })
  const isEditor = /^\/media\/[^/]+$/.test(pathname)
  const [createOpen, setCreateOpen] = useState(false)

  useEffect(() => {
    if (isEditor) return
    const requested = (location.search as Record<string, unknown>).new
    if (requested !== '1' && requested !== 1) return
    // eslint-disable-next-line react-hooks/set-state-in-effect -- latch the one-shot create query like the other hub routes
    setCreateOpen(true)
    void navigate({ to: '/media', replace: true })
  }, [isEditor, location.search, navigate])

  if (isEditor) return <Outlet />
  return (
    <>
      <MediaListView />
      <CreateMediaDialog
        open={createOpen}
        onOpenChange={setCreateOpen}
        onCreated={(mediaId) => {
          setCreateOpen(false)
          void navigate({ to: '/media/$mediaId', params: { mediaId }, search: emptyEditorReturnSearch() })
        }}
      />
    </>
  )
}
