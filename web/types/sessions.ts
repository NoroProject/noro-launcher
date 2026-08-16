/** Состояние лаунчера игрока — для карточки в админке. */
export interface LauncherStatus {
  online: boolean
  version: string | null
  platform: string | null
  last_seen_at: string | null
  current_version: string | null
  outdated: boolean
  open_integrity_flags: number
}

/** Активная сессия. */
export interface SessionRow {
  id: string
  scope: string
  created_at: string
  expires_at: string
  /** Кем открыта, если это impersonation. */
  impersonated_by: string | null
  /** Та ли это сессия, из которой пришёл запрос. */
  current: boolean
}
