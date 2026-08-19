/**
 * Тексты наказаний. Поля повторяют `ModerationMessages` на мастере — добавлять
 * шаблон надо в обоих местах, иначе панель его просто не покажет.
 */
export interface ModerationMessages {
  ban_permanent: string
  ban_temporary: string
  server_ban_permanent: string
  server_ban_temporary: string
  mute_permanent: string
  mute_temporary: string
  mute_actionbar_permanent: string
  mute_actionbar_temporary: string
  warn_actionbar: string
  warn_notice: string
  broadcast: string
  actor_receipt: string
  reason_by_rule: string
  no_account: string
  no_access: string
  maintenance: string
}

interface MessageField {
  key: keyof ModerationMessages
  /** Ключи каталога: подписи живут там же, где остальной интерфейс. */
  label: string
  hint: string
  rows: number
}

/**
 * Порядок полей — порядок разговора с игроком: сперва то, что выкидывает с
 * сервера, потом чат, потом объявления.
 */
export const MODERATION_MESSAGE_FIELDS: MessageField[] = [
  { key: 'ban_permanent', label: 'admin-moderation-ban-perm', hint: 'admin-moderation-ban-perm-hint', rows: 4 },
  { key: 'ban_temporary', label: 'admin-moderation-ban-temp', hint: 'admin-moderation-ban-temp-hint', rows: 4 },
  {
    key: 'server_ban_permanent',
    label: 'admin-moderation-sban-perm',
    hint: 'admin-moderation-sban-perm-hint',
    rows: 4,
  },
  {
    key: 'server_ban_temporary',
    label: 'admin-moderation-sban-temp',
    hint: 'admin-moderation-sban-temp-hint',
    rows: 4,
  },
  { key: 'mute_permanent', label: 'admin-moderation-mute-perm', hint: 'admin-moderation-mute-perm-hint', rows: 2 },
  { key: 'mute_temporary', label: 'admin-moderation-mute-temp', hint: 'admin-moderation-mute-temp-hint', rows: 2 },
  {
    key: 'mute_actionbar_permanent',
    label: 'admin-moderation-mute-bar-perm',
    hint: 'admin-moderation-mute-bar-hint',
    rows: 1,
  },
  {
    key: 'mute_actionbar_temporary',
    label: 'admin-moderation-mute-bar-temp',
    hint: 'admin-moderation-mute-bar-hint',
    rows: 1,
  },
  { key: 'warn_actionbar', label: 'admin-moderation-warn-bar', hint: 'admin-moderation-mute-bar-hint', rows: 1 },
  { key: 'warn_notice', label: 'admin-moderation-warn', hint: 'admin-moderation-warn-hint', rows: 2 },
  { key: 'broadcast', label: 'admin-moderation-broadcast', hint: 'admin-moderation-broadcast-hint', rows: 2 },
  { key: 'actor_receipt', label: 'admin-moderation-receipt', hint: 'admin-moderation-receipt-hint', rows: 2 },
  { key: 'reason_by_rule', label: 'admin-moderation-reason', hint: 'admin-moderation-reason-hint', rows: 1 },
  { key: 'no_account', label: 'admin-moderation-no-account', hint: 'admin-moderation-no-account-hint', rows: 4 },
  { key: 'no_access', label: 'admin-moderation-no-access', hint: 'admin-moderation-no-access-hint', rows: 4 },
  { key: 'maintenance', label: 'admin-moderation-maintenance', hint: 'admin-moderation-maintenance-hint', rows: 4 },
]
