/** Строка `restart_schedules` на мастере. */
export interface RestartSchedule {
  id: string
  game_server_id: string
  /** Сырой cron — для редких случаев вроде «только по будням». */
  cron_expr?: string | null
  /** Время суток в `HH:MM`, например `["05:00", "17:00"]`. */
  at_times?: string[] | null
  interval_minutes?: number | null
  /** За сколько минут предупредить игроков. */
  notice_minutes: number
  /** `warn_and_go` — рестарт в назначенное время; `defer` — ждать нулевого онлайна. */
  online_policy: string
  /** Докуда откладывать при `defer`. */
  max_defer_minutes: number
  active: boolean
  last_run_at?: string | null
  next_run_at?: string | null
  created_at: string
}

/** Тело `POST /api/admin/restarts`. Задать надо ровно один из трёх способов. */
export interface NewRestartSchedule {
  game_server_id: string
  cron_expr?: string
  at_times?: string[]
  interval_minutes?: number
  notice_minutes: number
  online_policy: string
  max_defer_minutes: number
}
