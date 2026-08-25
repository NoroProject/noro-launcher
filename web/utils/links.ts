/**
 * Пути сайта и внешние ссылки — одним местом.
 *
 * Путь, набранный строкой в разметке, живёт до первого переименования
 * страницы: найти его потом можно только текстовым поиском по всему вебу, а
 * промахнувшаяся ссылка не падает, а молча ведёт на 404. Здесь же каждый путь
 * записан один раз, а разделы админки заодно видны списком.
 *
 * Всё вызывается как функция, даже когда параметров нет: `link.rules()` и
 * `link.rule(code)` рядом читаются одинаково, а половина констант вперемешку с
 * половиной функций — нет.
 */

/** Разделы сайта, открытые всем. */
export const link = {
  home: () => '/',
  servers: () => '/servers',
  rules: () => '/rules',
  /** Пункт свода: тот же якорь, что ставит агент в ссылке из бана. */
  rule: (code: string) => `/rules#rule-${code}`,
  skin: () => '/skin',
  setup: () => '/setup',
  oauth2Authorize: () => '/oauth2/authorize',

  /** `next` — куда вернуть после входа; кодируется здесь, а не на месте. */
  login: (next?: string) => (next ? `/login?next=${encodeURIComponent(next)}` : '/login'),

  cabinet: () => '/cabinet',
  cabinetSettings: () => '/cabinet/settings',
  cabinetPunishments: () => '/cabinet/punishments',
  cabinetApps: () => '/cabinet/apps',
}

/** Админка. Отдельным объектом: у неё свой вход и свои права. */
export const adminLink = {
  root: () => '/admin',
  apps: () => '/admin/apps',
  audit: () => '/admin/audit',
  backup: () => '/admin/backup',
  automod: () => '/admin/automod',
  blocklist: () => '/admin/blocklist',
  capes: () => '/admin/capes',
  integrity: () => '/admin/integrity',
  moderation: () => '/admin/moderation',
  mods: () => '/admin/mods',
  reports: () => '/admin/reports',
  rules: () => '/admin/rules',
  settings: () => '/admin/settings',
  support: () => '/admin/support',
  translations: () => '/admin/translations',
  wrapper: () => '/admin/wrapper',

  cases: () => '/admin/cases',
  case: (id: string) => `/admin/cases/${id}`,

  clients: () => '/admin/clients',
  client: (id: string) => `/admin/clients/${id}`,
  build: (id: string, buildId: string) => `/admin/clients/${id}/build/${buildId}`,
  buildMods: (id: string, buildId: string) => `/admin/clients/${id}/build/${buildId}/mods`,
  gameServer: (id: string, gameServerId: string) => `/admin/clients/${id}/game/${gameServerId}`,

  launcher: () => '/admin/launcher',
  launcherTokens: () => '/admin/launcher/tokens',

  news: () => '/admin/news',
  newsItem: (id: string) => `/admin/news/${id}`,
  roles: () => '/admin/roles',
  role: (id: string) => `/admin/roles/${id}`,
  users: () => '/admin/users',
  user: (id: string) => `/admin/users/${id}`,
}

/** Чужие сайты. Хостов у мода два, и вспоминать их формат каждый раз незачем. */
export const externalLink = {
  modrinth: (slug: string) => `https://modrinth.com/mod/${slug}`,
  curseforge: (slug: string) => `https://www.curseforge.com/minecraft/mc-mods/${slug}`,

  /**
   * Страница мода по провайдеру. Готовый адрес возвращается как есть: в базе
   * попадаются и полные ссылки вместо идентификатора.
   */
  mod: (provider: string | null | undefined, projectId: string | null | undefined) => {
    if (!projectId) return ''
    if (projectId.startsWith('http://') || projectId.startsWith('https://')) return projectId
    return provider?.toLowerCase() === 'curseforge'
      ? externalLink.curseforge(projectId)
      : externalLink.modrinth(projectId)
  },
}
