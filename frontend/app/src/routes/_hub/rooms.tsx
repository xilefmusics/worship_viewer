import { createFileRoute, useLocation, useNavigate } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { CreateRoomDialog } from '@/components/room/CreateRoomDialog'
import { RoomsList } from '@/components/room/RoomsList'
import { useWritableTeams } from '@/hooks/useWritableTeams'
import { isRoomsV2Enabled } from '@/lib/feature-flags'

export const Route = createFileRoute('/_hub/rooms')({ component: RoomsRoute })

export function RoomsRoute() {
  const roomsEnabled = isRoomsV2Enabled()
  const { t } = useTranslation()
  const location = useLocation()
  const navigate = useNavigate()
  const [createOpen, setCreateOpen] = useState(false)
  const { teams, user } = useWritableTeams('roomCreate', roomsEnabled)

  useEffect(() => {
    if (!roomsEnabled) return
    const raw = (location.search as Record<string, unknown>).new
    if (raw !== '1' && raw !== 1) return
    // eslint-disable-next-line react-hooks/set-state-in-effect -- latch the hub FAB query state into the dialog
    setCreateOpen(true)
    void navigate({ to: '/rooms', replace: true })
  }, [location.search, navigate, roomsEnabled])

  if (!roomsEnabled) {
    return <p className="p-6 text-center">{t('rooms.comingSoon')}</p>
  }

  return (
    <>
      <RoomsList />
      <CreateRoomDialog
        open={createOpen}
        onOpenChange={setCreateOpen}
        teams={teams}
        userId={user?.id}
        onCreated={(roomId) => {
          window.location.assign(`/rooms/${encodeURIComponent(roomId)}`)
        }}
      />
    </>
  )
}
