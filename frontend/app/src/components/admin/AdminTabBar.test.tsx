import { render, screen } from '@testing-library/react'
import type { AnchorHTMLAttributes, ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'

import { AdminTabBar } from '@/components/admin/AdminTabBar'

vi.mock('@tanstack/react-router', async () => {
  const { forwardRef } = await import('react')
  return {
    Link: forwardRef<
      HTMLAnchorElement,
      AnchorHTMLAttributes<HTMLAnchorElement> & {
        children: ReactNode
        to: string
        search?: unknown
      }
    >(function MockLink({ children, to, search, onClick, ...props }, ref) {
      void search
      return (
        <a
          ref={ref}
          href={to}
          onClick={(event) => {
            onClick?.(event)
            event.preventDefault()
          }}
          {...props}
        >
          {children}
        </a>
      )
    }),
    useRouterState: ({
      select,
    }: {
      select: (state: { location: { pathname: string } }) => unknown
    }) => select({ location: { pathname: '/admin/users' } }),
  }
})
vi.mock('react-i18next', () => ({ useTranslation: () => ({ t: (key: string) => key }) }))

describe('Admin tab bar', () => {
  it('puts leave first, matches hub tab sizing, and marks the users destination active', () => {
    render(<AdminTabBar />)
    const nav = screen.getByRole('navigation', { name: 'adminNav.aria' })
    const navLinks = nav.querySelectorAll('a')
    expect(navLinks).toHaveLength(3)
    expect(navLinks[0]).toHaveTextContent('adminNav.leave')
    expect(screen.getByRole('link', { name: 'adminNav.leave' })).toHaveAttribute('href', '/collections')
    expect(screen.getByRole('link', { name: 'adminNav.leave' })).not.toHaveAttribute('aria-current')
    expect(screen.getByRole('link', { name: 'adminNav.users' })).toHaveAttribute('aria-current', 'page')
    expect(screen.getByRole('link', { name: 'adminNav.metrics' })).toHaveAttribute('href', '/admin/metrics')
    expect(screen.getByRole('link', { name: 'adminNav.metrics' })).not.toHaveAttribute('aria-current')
    for (const link of navLinks) {
      expect(link).toHaveClass('[aspect-ratio:var(--hub-tab-aspect)]')
    }
  })
})
