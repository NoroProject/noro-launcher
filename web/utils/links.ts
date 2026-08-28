/**
 * Site paths and external links in one place. A path typed inline in markup
 * survives until the page is renamed, and a broken link doesn't fail — it
 * quietly serves a 404.
 *
 * Everything is a function even with no parameters: `link.rules()` and
 * `link.rule(code)` read the same side by side.
 */

/** Pages open to everyone. */
export const link = {
  home: () => '/',
  servers: () => '/servers',
  rules: () => '/rules',
  /** Same anchor the agent puts in the link from a ban message. */
  rule: (code: string) => `/rules#rule-${code}`,
  skin: () => '/skin',
  setup: () => '/setup',
  oauth2Authorize: () => '/oauth2/authorize',

  /** `next` is where to go after sign-in; encoded here, not at each caller. */
  login: (next?: string) => (next ? `/login?next=${encodeURIComponent(next)}` : '/login'),

  cabinet: () => '/cabinet',
  cabinetSettings: () => '/cabinet/settings',
  cabinetPunishments: () => '/cabinet/punishments',
  cabinetApps: () => '/cabinet/apps',
}

/** Admin. Its own object: separate entry point, separate permissions. */
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

/** Third-party sites. A mod can live on either host. */
export const externalLink = {
  modrinth: (slug: string) => `https://modrinth.com/mod/${slug}`,
  curseforge: (slug: string) => `https://www.curseforge.com/minecraft/mc-mods/${slug}`,

  /**
   * Mod page for a provider. A ready URL comes back untouched — the database
   * holds full links in the id column in places.
   */
  mod: (provider: string | null | undefined, projectId: string | null | undefined) => {
    if (!projectId) return ''
    if (projectId.startsWith('http://') || projectId.startsWith('https://')) return projectId
    return provider?.toLowerCase() === 'curseforge'
      ? externalLink.curseforge(projectId)
      : externalLink.modrinth(projectId)
  },
}
