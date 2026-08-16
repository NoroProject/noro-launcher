/** Настройка инстанса. */
export interface SettingItem {
  key: string
  /** Переменная окружения, которая её перекрывает. */
  env: string
  value: string
  /** Значение пришло из окружения — правка в БД его не перекроет. */
  from_env: boolean
}

export interface SettingsResponse {
  settings: SettingItem[]
  /** Какие секреты видны мастеру. Значения не отдаются никогда. */
  secrets: Record<string, boolean>
}

export interface DiagnosticCheck {
  id: string
  title: string
  level: 'ok' | 'warn' | 'fail'
  detail: string
}

/** Человеческие названия и группы для полей настроек. */
export const SETTING_LABELS: Record<string, { label: string; hint?: string; section: string }> = {
  instance_name: { label: 'Instance name', section: 'General' },
  public_url: {
    label: 'API URL',
    hint: 'Ends up in every manifest a player downloads.',
    section: 'General',
  },
  web_url: {
    label: 'Site URL',
    hint: 'Passkeys are bound to this domain permanently.',
    section: 'General',
  },
  allowed_origins: {
    label: 'Allowed CORS origins',
    hint: 'Comma-separated. Empty means any origin is accepted.',
    section: 'Auth',
  },
  discord_client_id: { label: 'Discord client ID', section: 'Auth' },
  files_cdn_url: { label: 'CDN URL for files', section: 'Storage' },
  github_repo: { label: 'GitHub repository', section: 'Integrations' },
  github_ref: { label: 'GitHub branch', section: 'Integrations' },
  launcher_repo: { label: 'Local launcher checkout', section: 'Integrations' },
}

export const SECTIONS = ['General', 'Auth', 'Storage', 'Integrations'] as const
