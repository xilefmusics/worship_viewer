import { createFileRoute } from '@tanstack/react-router'

import { AdminUsersView } from '@/components/admin/AdminUsersView'

export const Route = createFileRoute('/_hub/admin/users')({
  component: AdminUsersView,
})
