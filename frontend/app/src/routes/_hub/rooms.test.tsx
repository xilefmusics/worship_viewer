import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { RoomsRoute } from '@/routes/_hub/rooms'

const flags = vi.hoisted(() => ({ enabled: false }))

vi.mock('@/lib/feature-flags', () => ({
  isRoomsV2Enabled: () => flags.enabled,
}))
vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}))
vi.mock('@tanstack/react-router', () => ({
  createFileRoute: () => (opts: { component: unknown }) => opts,
  useLocation: () => ({ search: {} }),
  useNavigate: () => vi.fn(),
}))
vi.mock('@/hooks/useWritableTeams', () => ({
  useWritableTeams: () => ({ teams: [], user: undefined }),
}))
vi.mock('@/components/room/RoomsList', () => ({
  RoomsList: () => <div data-testid="rooms-list" />,
}))
vi.mock('@/components/room/CreateRoomDialog', () => ({
  CreateRoomDialog: () => <div data-testid="create-room-dialog" />,
}))

describe('RoomsRoute', () => {
  it('shows coming soon when rooms v2 is disabled', () => {
    flags.enabled = false
    render(<RoomsRoute />)
    expect(screen.getByText('rooms.comingSoon')).toBeInTheDocument()
    expect(screen.queryByTestId('rooms-list')).not.toBeInTheDocument()
    expect(screen.queryByTestId('create-room-dialog')).not.toBeInTheDocument()
  })

  it('shows the rooms list when rooms v2 is enabled', () => {
    flags.enabled = true
    render(<RoomsRoute />)
    expect(screen.getByTestId('rooms-list')).toBeInTheDocument()
    expect(screen.getByTestId('create-room-dialog')).toBeInTheDocument()
    expect(screen.queryByText('rooms.comingSoon')).not.toBeInTheDocument()
  })
})
