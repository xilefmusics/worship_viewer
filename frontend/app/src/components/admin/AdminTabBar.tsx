import { Link, useRouterState } from '@tanstack/react-router'
import { motion, useReducedMotion } from 'motion/react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'

import { IconHubAdminLeave, IconHubAdminMetrics, IconHubAdminUsers } from '@/components/icons/hub-tab-icons'
import { HUB_TAB_LABEL_CLASS } from '@/components/hub/hub-list-styles'
import { cn } from '@/lib/utils'

type AdminTabTo = '/collections' | '/admin/users' | '/admin/metrics'

const tabs = [
  { to: '/collections' as const, labelKey: 'adminNav.leave' as const, Icon: IconHubAdminLeave },
  { to: '/admin/users' as const, labelKey: 'adminNav.users' as const, Icon: IconHubAdminUsers },
  { to: '/admin/metrics' as const, labelKey: 'adminNav.metrics' as const, Icon: IconHubAdminMetrics },
] satisfies ReadonlyArray<{
  to: AdminTabTo
  labelKey: 'adminNav.leave' | 'adminNav.users' | 'adminNav.metrics'
  Icon: typeof IconHubAdminUsers
}>

export function AdminTabBar() {
  const { t } = useTranslation()
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const [hoveredTab, setHoveredTab] = useState<AdminTabTo | null>(null)
  const reduceMotion = useReducedMotion()

  const barSpring =
    reduceMotion
      ? { duration: 0 }
      : { type: 'spring' as const, stiffness: 380, damping: 34, mass: 0.92 }

  return (
    <motion.nav
      layout={!reduceMotion}
      initial={false}
      transition={barSpring}
      className={cn(
        'flex h-[3.6rem] w-full min-w-0 flex-1 items-stretch justify-between rounded-full border border-[var(--color-border)] bg-[var(--color-surface)] p-[0.18rem]',
        'shadow-[var(--shadow-elevated)]',
      )}
      aria-label={t('adminNav.aria')}
    >
      {tabs.map(({ to, labelKey, Icon }) => {
        const active =
          to === '/collections' ? false : pathname === to || pathname.startsWith(`${to}/`)
        return (
          <Link
            key={to}
            to={to}
            onMouseEnter={() => setHoveredTab(to)}
            onMouseLeave={() => setHoveredTab(null)}
            aria-current={active ? 'page' : undefined}
            className={cn(
              'relative flex h-full shrink-0 flex-none flex-col items-center justify-center gap-0.5 rounded-full px-1 text-center [aspect-ratio:var(--hub-tab-aspect)]',
              HUB_TAB_LABEL_CLASS,
              active
                ? 'text-[var(--color-primary-foreground)]'
                : 'text-[var(--color-muted-foreground)] hover:bg-[var(--color-muted)]',
            )}
          >
            {active ? (
              <motion.span
                layoutId="admin-tab-pill"
                className="absolute inset-0 rounded-full bg-[var(--color-primary)]"
                aria-hidden
                transition={
                  reduceMotion
                    ? { duration: 0 }
                    : { type: 'spring', stiffness: 460, damping: 38, mass: 0.9 }
                }
              />
            ) : null}
            <span className="relative z-10 flex min-h-0 w-full flex-col items-center justify-center gap-0.5">
              <Icon isHovered={hoveredTab === to} />
              <span className="line-clamp-1 w-full min-w-0 px-0.5 [overflow-wrap:anywhere]">
                {t(labelKey)}
              </span>
            </span>
          </Link>
        )
      })}
    </motion.nav>
  )
}
