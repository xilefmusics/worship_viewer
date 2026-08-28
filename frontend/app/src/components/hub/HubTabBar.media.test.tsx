import { render, screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'

import { HubTabBar } from '@/components/hub/HubTabBar'

vi.mock('@tanstack/react-router', () => ({
  Link: ({ children, to }: { children: ReactNode; to: string }) => <a href={to}>{children}</a>,
  useRouterState: ({ select }: { select: (state: { location: { pathname: string } }) => unknown }) => select({ location: { pathname: '/media' } }),
}))
vi.mock('react-i18next', () => ({ useTranslation: () => ({ t: (key: string) => key }) }))
describe('Media primary navigation', () => {
  it('keeps Media absent from the primary hub tab bar', () => {
    render(<HubTabBar />)
    expect(screen.queryByRole('link', { name: /media/i })).not.toBeInTheDocument()
    expect(screen.getAllByRole('link')).toHaveLength(4)
  })
})
