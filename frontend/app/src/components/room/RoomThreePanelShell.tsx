import { useEffect, useLayoutEffect, useRef, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'

import { PLAYER_TOC_WIDTH_CLASS } from '@/lib/player/player-chrome'
import { cn } from '@/lib/utils'

type Panel = 'queue' | 'player' | 'details'

type Props = {
  queue: ReactNode
  player: ReactNode
  details: ReactNode
  desktopOverlay?: boolean
}

const panels: Panel[] = ['queue', 'player', 'details']
const PANEL_WHEEL_THRESHOLD_PX = 16
const PANEL_WHEEL_IDLE_MS = 100

export function RoomThreePanelShell({ queue, player, details, desktopOverlay = false }: Props) {
  const { t } = useTranslation()
  const viewportRef = useRef<HTMLDivElement>(null)
  const wheelDeltaRef = useRef(0)
  const wheelDirectionRef = useRef<number | null>(null)
  const wheelLockTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  useLayoutEffect(() => {
    const viewport = viewportRef.current
    if (!viewport || viewport.scrollWidth <= viewport.clientWidth) return
    viewport.scrollLeft = viewport.clientWidth
  }, [])

  useEffect(() => {
    return () => {
      if (wheelLockTimeoutRef.current != null) {
        clearTimeout(wheelLockTimeoutRef.current)
      }
    }
  }, [])

  function armWheelLock() {
    if (wheelLockTimeoutRef.current != null) {
      clearTimeout(wheelLockTimeoutRef.current)
    }
    wheelLockTimeoutRef.current = setTimeout(() => {
      wheelLockTimeoutRef.current = null
      wheelDirectionRef.current = null
      wheelDeltaRef.current = 0
    }, PANEL_WHEEL_IDLE_MS)
  }

  function onWheel(event: React.WheelEvent<HTMLDivElement>) {
    const deltaX = event.deltaX
    if (deltaX === 0 || Math.abs(deltaX) <= Math.abs(event.deltaY)) return

    // Trackpad horizontal scrolling over a nested player surface is not
    // consistently chained to this scrollport by desktop browsers. Consume
    // the gesture here and snap to the adjacent room panel instead.
    event.preventDefault()
    if (wheelLockTimeoutRef.current != null) {
      // Trackpad momentum arrives as a stream of wheel events. Keep the
      // gesture locked until that stream has been quiet for a short period.
      armWheelLock()
      return
    }

    const direction = Math.sign(deltaX)
    if (wheelDirectionRef.current !== direction) {
      wheelDeltaRef.current = 0
      wheelDirectionRef.current = direction
    }
    wheelDeltaRef.current += deltaX
    if (Math.abs(wheelDeltaRef.current) < PANEL_WHEEL_THRESHOLD_PX) return

    const viewport = event.currentTarget
    if (viewport.clientWidth <= 0) return
    const currentIndex = Math.max(
      0,
      Math.min(panels.length - 1, Math.round(viewport.scrollLeft / viewport.clientWidth)),
    )
    const nextIndex = Math.max(
      0,
      Math.min(panels.length - 1, currentIndex + (direction > 0 ? 1 : -1)),
    )
    wheelDeltaRef.current = 0
    if (nextIndex === currentIndex) return

    viewport.scrollTo({ left: nextIndex * viewport.clientWidth, behavior: 'smooth' })
    armWheelLock()
  }

  const panelLabel = (panel: Panel) => t(`rooms.panel.${panel}`)

  return (
    <div className="flex h-dvh min-h-0 flex-col overflow-hidden bg-[var(--color-background)]">
      <div
        ref={viewportRef}
        className={cn(
          'flex min-h-0 min-w-0 flex-1 snap-x snap-mandatory touch-pan-x overflow-x-auto overscroll-x-contain',
          desktopOverlay
            ? 'md:relative md:grid md:grid-cols-1 md:overflow-hidden md:overscroll-none md:snap-none'
            : 'md:grid md:grid-cols-[minmax(16rem,20rem)_minmax(0,1fr)_minmax(16rem,20rem)] md:overflow-hidden md:snap-none',
        )}
        onWheel={onWheel}
      >
        <section
          className={cn(
            'min-h-0 min-w-full snap-center md:min-w-0',
            desktopOverlay && `md:absolute md:inset-y-0 md:left-0 md:z-10 md:flex ${PLAYER_TOC_WIDTH_CLASS} md:shadow-[var(--shadow-elevated)]`,
          )}
          aria-label={panelLabel('queue')}
        >
          {queue}
        </section>
        <main className="min-h-0 min-w-full snap-center overflow-hidden md:min-w-0" aria-label={panelLabel('player')}>
          {player}
        </main>
        <section
          className={cn(
            'min-h-0 min-w-full snap-center md:min-w-0',
            desktopOverlay && `md:absolute md:inset-y-0 md:right-0 md:z-10 md:flex ${PLAYER_TOC_WIDTH_CLASS} md:shadow-[var(--shadow-elevated)]`,
          )}
          aria-label={panelLabel('details')}
        >
          {details}
        </section>
      </div>
    </div>
  )
}
