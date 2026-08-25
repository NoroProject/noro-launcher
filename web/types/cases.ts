/** Дела: очередь разборов и всё, что в карточке одного разбора. */

export interface CaseRow {
  id: string
  /** Порядковый номер дела; печатается через `caseNumber()`. */
  number: number
  target_id: string
  target_name: string | null
  game_server_id: string | null
  server_name: string | null
  status: 'open' | 'in_review' | 'resolved' | 'rejected'
  claimed_by: string | null
  claimed_by_name: string | null
  opened_at: string
  resolved_at: string | null
  verdict: 'confirmed' | 'rejected' | 'insufficient' | null
  rule_code: string | null
  reports_count: number
  reporters_count: number
  last_report_at: string | null
}

export interface CaseEvent {
  id: string
  at: string
  actor_label: string
  /** Откуда действовали: с сайта, из игры или это сделала система. */
  source: 'web' | 'game' | 'system'
  kind: string
  payload: Record<string, unknown>
}

export interface CaseMessage {
  id: string
  at: string
  sender_name: string
  channel: 'public' | 'local' | 'private' | 'command'
  content: string
}

export interface CaseReport {
  id: string
  reporter_id: string
  reporter_name: string | null
  reason: string
  world: string | null
  x: number | null
  y: number | null
  z: number | null
  created_at: string
}

/** Сколько жалоб человека подтвердилось — вес его слова в очереди. */
export interface ReporterStats {
  total: number
  confirmed: number
  rejected: number
}

export interface CasePunishment {
  id: string
  kind: string
  reason: string
  created_at: string
  expires_at: string | null
  revoked_at: string | null
  rule_code: string | null
}

export interface CaseDetail {
  case: CaseRow
  reports: CaseReport[]
  events: CaseEvent[]
  punishments: CasePunishment[]
  messages: CaseMessage[]
  /** Есть ли право на срез чата: без него `messages` приходит пустым. */
  chat_allowed: boolean
  reporters: Record<string, ReporterStats>
}
