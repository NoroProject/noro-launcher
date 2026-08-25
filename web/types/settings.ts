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
  /** У каких залитых картинок есть прозрачные места. */
  transparency: Record<string, boolean>
}

export interface DiagnosticCheck {
  id: string
  title: string
  level: 'ok' | 'warn' | 'fail'
  detail: string
}

/** Человеческие названия и группы для полей настроек. */
export const SETTING_LABELS: Record<string, { label: string; hint?: string; section: string }> = {
  instance_name: {
    label: 'Instance name',
    hint: 'Shown in the site header and as the main page heading.',
    section: 'Branding',
  },
  logo_url: {
    label: 'Logo URL',
    hint: 'Site header icon. Empty falls back to the icon shipped with the build.',
    section: 'Branding',
  },
  hero_image_url: {
    label: 'Hero illustration URL',
    hint: 'Main page character render image URL (updated instantly without restart when uploaded).',
    section: 'Branding',
  },
  login_image_url: {
    label: 'Login page image URL',
    hint: 'Shown beside the sign-in form. Animated GIF/WebP keep their animation and transparency.',
    section: 'Branding',
  },
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
    section: 'General',
  },
  files_cdn_url: { label: 'CDN URL for files', section: 'Storage' },
  github_repo: { label: 'GitHub repository', section: 'Integrations' },
  github_ref: { label: 'GitHub branch', section: 'Integrations' },
  launcher_repo: { label: 'Local launcher checkout', section: 'Integrations' },
}

/**
 * Вкладки страницы настроек.
 *
 * `section` есть у тех, за которыми стоят поля из `SETTING_LABELS`; у остальных
 * своё содержимое — способы входа со своей ручкой и сводка состояния инстанса.
 */
export interface SettingsTab {
  id: string
  icon: string
  /** Ключ перевода для подписи вкладки. */
  label: string
  section?: string
}

export const SETTINGS_TABS: SettingsTab[] = [
  { id: 'branding', icon: 'i-lucide-palette', label: 'admin-settings-tab-branding', section: 'Branding' },
  { id: 'general', icon: 'i-lucide-sliders-horizontal', label: 'admin-settings-tab-general', section: 'General' },
  { id: 'sign-in', icon: 'i-lucide-log-in', label: 'admin-settings-tab-sign-in' },
  { id: 'storage', icon: 'i-lucide-hard-drive', label: 'admin-settings-tab-storage', section: 'Storage' },
  { id: 'integrations', icon: 'i-lucide-plug', label: 'admin-settings-tab-integrations', section: 'Integrations' },
  { id: 'health', icon: 'i-lucide-activity', label: 'admin-settings-tab-health' },
]

/** Иллюстрации инстанса — все через одну ручку загрузки, отличаются ключом. */
export const BRANDING_IMAGES = [
  { key: 'logo_url', title: 'admin-settings-logo-title', desc: 'admin-settings-logo-desc' },
  { key: 'hero_image_url', title: 'admin-settings-hero-title', desc: 'admin-settings-hero-desc' },
  { key: 'login_image_url', title: 'admin-settings-login-image-title', desc: 'admin-settings-login-image-desc' },
] as const
