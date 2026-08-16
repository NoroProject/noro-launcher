/** Запись журнала админских действий. */
export interface AuditRow {
  id: number
  at: string
  /** Пусто, если аккаунт удалён или действовал admin-токен — тогда смотри actor_label. */
  actor_id: string | null
  actor_label: string
  /** Точечная нотация: user.ban, role.permission.add, build.publish. */
  action: string
  target_kind: string | null
  target_id: string | null
  details: Record<string, unknown>
  ip: string | null
}
