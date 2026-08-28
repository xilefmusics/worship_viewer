import { createFileRoute, redirect } from '@tanstack/react-router'

import { AdminDashboardView } from '@/components/admin/AdminDashboardView'
import {
  formatAdminDateInputValue,
  resolveAdminDateRangeFromStrings,
  resolveAdminQuickRange,
} from '@/lib/admin-dashboard'

function defaultAdminRange(): { start: string; end: string } {
  const range = resolveAdminQuickRange('30d')
  return {
    start: formatAdminDateInputValue(range.start),
    end: formatAdminDateInputValue(range.end),
  }
}

function normalizeDateSearch(start: string, end: string): { start: string; end: string } | null {
  const parsed = resolveAdminDateRangeFromStrings(start, end)
  if (!parsed) return null
  const ordered =
    parsed.start.getTime() <= parsed.end.getTime()
      ? parsed
      : { start: parsed.end, end: parsed.start }
  return {
    start: formatAdminDateInputValue(ordered.start),
    end: formatAdminDateInputValue(ordered.end),
  }
}

export const Route = createFileRoute('/_hub/admin/metrics')({
  beforeLoad: ({ search }) => {
    if (typeof search.start !== 'string' || typeof search.end !== 'string') {
      throw redirect({ to: '/admin/metrics', search: defaultAdminRange(), replace: true })
    }

    const normalized = normalizeDateSearch(search.start, search.end)
    if (!normalized) {
      throw redirect({ to: '/admin/metrics', search: defaultAdminRange(), replace: true })
    }

    if (normalized.start !== search.start || normalized.end !== search.end) {
      throw redirect({ to: '/admin/metrics', search: normalized, replace: true })
    }
  },
  validateSearch: (search: Record<string, unknown>) => ({
    start: typeof search.start === 'string' ? search.start : undefined,
    end: typeof search.end === 'string' ? search.end : undefined,
  }),
  component: AdminMetricsRoute,
})

function AdminMetricsRoute() {
  const { start, end } = Route.useSearch()
  if (!start || !end) return null
  return <AdminDashboardView startDate={start} endDate={end} />
}
