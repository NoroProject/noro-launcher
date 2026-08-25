/** OAuth2-приложение глазами его владельца. */
export interface MyApp {
  id: string
  client_id: string
  name: string
  description: string | null
  icon_url: string | null
  /** Адреса возврата, по одному в строке. */
  redirect_uris: string
  status: AppStatus
  /** Что оператор ответил при отказе или блокировке. */
  review_note: string | null
  allowed_scopes: string[]
  is_official: boolean
  created_at: string
  updated_at: string
}

/** Приложение глазами оператора: плюс владелец и охват. */
export interface AdminApp extends Omit<MyApp, 'redirect_uris'> {
  redirect_uris: string[]
  owner: { id: string; username: string } | null
  authorized_users: number
  reviewed_at: string | null
}

export type AppStatus = 'pending' | 'approved' | 'rejected' | 'suspended'

export interface ScopeInfo {
  name: string
  title: string
  tier: 'basic' | 'privileged'
}

/**
 * Подпись статуса и цвет для бейджа.
 *
 * Одним местом на кабинет и админку: статусов четыре, и расходиться им незачем.
 */
export const APP_STATUS_META: Record<AppStatus, { label: string; class: string }> = {
  pending: {
    label: 'app-status-pending',
    class: 'bg-[color-mix(in_srgb,var(--noro-cream)_16%,transparent)] text-[var(--noro-cream)]',
  },
  approved: {
    label: 'app-status-approved',
    class: 'bg-[color-mix(in_srgb,var(--noro-green)_18%,transparent)] text-[var(--noro-green)]',
  },
  rejected: {
    label: 'app-status-rejected',
    class: 'bg-[color-mix(in_srgb,var(--noro-danger)_18%,transparent)] text-[var(--noro-danger)]',
  },
  suspended: {
    label: 'app-status-suspended',
    class: 'bg-[color-mix(in_srgb,var(--noro-danger)_18%,transparent)] text-[var(--noro-danger)]',
  },
}
