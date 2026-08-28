import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { AnchorHTMLAttributes, ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'

import { AdminLayout, AdminMobileNav } from '@/components/admin/AdminNavigation'

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

describe('Admin navigation', () => {
  it('marks the users destination active in the desktop sidebar', () => {
    render(<AdminLayout><p>Content</p></AdminLayout>)
    const navLinks = screen.getByRole('navigation', { name: 'adminNav.aria' }).querySelectorAll('a')
    expect(navLinks[0]).toHaveTextContent('adminNav.users')
    expect(screen.getByRole('link', { name: 'adminNav.users' })).toHaveAttribute('aria-current', 'page')
    expect(screen.getByRole('link', { name: 'adminNav.metrics' })).not.toHaveAttribute('aria-current')
    expect(screen.getByRole('link', { name: 'adminNav.back' })).toHaveAttribute('href', '/collections')
  })

  it('opens the mobile drawer and closes it after navigation', async () => {
    const user = userEvent.setup()
    render(<AdminMobileNav />)
    await user.click(screen.getByRole('button', { name: 'adminNav.open' }))
    const usersLink = screen.getByRole('link', { name: 'adminNav.users' })
    expect(usersLink).toBeVisible()
    await user.click(usersLink)
    await vi.waitFor(() => expect(screen.queryByRole('link', { name: 'adminNav.users' })).not.toBeInTheDocument())
  })
})
