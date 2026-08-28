import { Link, useRouterState } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'

import { UsersIcon } from '@/components/icons/lucide-animated/users-icon'
import { Button } from '@/components/ui/button'
import { Sheet, SheetClose, SheetContent, SheetTitle, SheetTrigger } from '@/components/ui/sheet'
import { cn } from '@/lib/utils'

function ChartIcon({ className }: { className?: string }) {
  return (
    <svg aria-hidden className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 3v18h18" />
      <path d="m7 16 4-5 4 3 5-7" />
    </svg>
  )
}

function MenuIcon() {
  return (
    <svg aria-hidden fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
      <path d="M4 6h16M4 12h16M4 18h16" />
    </svg>
  )
}

function ArrowLeftIcon({ className }: { className?: string }) {
  return (
    <svg aria-hidden className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="m15 18-6-6 6-6" />
    </svg>
  )
}

function AdminNavContent({ mobile = false }: { mobile?: boolean }) {
  const { t } = useTranslation()
  const pathname = useRouterState({ select: (state) => state.location.pathname })
  const links = [
    { to: '/admin/users' as const, label: t('adminNav.users'), icon: UsersIcon },
    { to: '/admin/metrics' as const, label: t('adminNav.metrics'), icon: ChartIcon },
  ]

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-[var(--color-border)] px-4 py-5">
        <p className="text-xs font-medium uppercase tracking-[0.18em] text-[var(--color-muted-foreground)]">{t('adminNav.kicker')}</p>
        <p className="mt-1 text-lg font-semibold">{t('adminNav.title')}</p>
      </div>
      <nav className="flex flex-1 flex-col gap-1 p-3" aria-label={t('adminNav.aria')}>
        {links.map(({ to, label, icon: Icon }) => {
          const active = pathname === to || pathname.startsWith(`${to}/`)
          const link = (
            <Link
              to={to}
              search={to === '/admin/metrics' ? undefined : {}}
              aria-current={active ? 'page' : undefined}
              className={cn(
                'flex h-10 items-center gap-3 rounded-md px-3 text-sm font-medium transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-primary)]',
                active
                  ? 'bg-[var(--color-primary)] text-[var(--color-primary-foreground)]'
                  : 'text-[var(--color-muted-foreground)] hover:bg-[var(--color-muted)] hover:text-[var(--color-foreground)]',
              )}
            >
              <Icon className="size-4" size={16} />
              {label}
            </Link>
          )
          return mobile ? <SheetClose asChild key={to}>{link}</SheetClose> : <div key={to}>{link}</div>
        })}
      </nav>
      <div className="border-t border-[var(--color-border)] p-3">
        {mobile ? (
          <SheetClose asChild>
            <Link to="/collections" className="flex h-10 items-center gap-3 rounded-md px-3 text-sm font-medium text-[var(--color-muted-foreground)] hover:bg-[var(--color-muted)] hover:text-[var(--color-foreground)]">
              <ArrowLeftIcon className="size-4" />
              {t('adminNav.back')}
            </Link>
          </SheetClose>
        ) : (
          <Link to="/collections" className="flex h-10 items-center gap-3 rounded-md px-3 text-sm font-medium text-[var(--color-muted-foreground)] hover:bg-[var(--color-muted)] hover:text-[var(--color-foreground)]">
            <ArrowLeftIcon className="size-4" />
            {t('adminNav.back')}
          </Link>
        )}
      </div>
    </div>
  )
}

export function AdminMobileNav() {
  const { t } = useTranslation()
  return (
    <Sheet>
      <SheetTrigger asChild>
        <Button type="button" size="icon" variant="outline" className="my-[0.36rem] size-[3.6rem] shrink-0 rounded-full shadow-[var(--shadow-elevated)] lg:hidden" aria-label={t('adminNav.open')}>
          <MenuIcon />
        </Button>
      </SheetTrigger>
      <SheetContent aria-describedby={undefined}>
        <SheetTitle className="sr-only">{t('adminNav.title')}</SheetTitle>
        <AdminNavContent mobile />
      </SheetContent>
    </Sheet>
  )
}

export function AdminLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="mx-auto flex w-full max-w-[96rem] min-w-0 items-start gap-4">
      <aside className="sticky top-0 hidden h-[calc(100dvh-6.6rem)] w-60 shrink-0 overflow-hidden rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] shadow-sm lg:block">
        <AdminNavContent />
      </aside>
      <section className="min-w-0 flex-1">{children}</section>
    </div>
  )
}
