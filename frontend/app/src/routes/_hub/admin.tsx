import { Outlet, createFileRoute } from '@tanstack/react-router'

import { AdminLayout } from '@/components/admin/AdminNavigation'
import { requireAdminSession } from '@/lib/auth-guard'

export const Route = createFileRoute('/_hub/admin')({
  beforeLoad: async ({ context }) => {
    await requireAdminSession(context)
  },
  component: AdminLayoutRoute,
})

function AdminLayoutRoute() {
  return (
    <AdminLayout>
      <Outlet />
    </AdminLayout>
  )
}
