export default defineNuxtRouteMiddleware(async (to) => {
  // /setup доступен без входа: аккаунтов на ненастроенном инстансе ещё нет.
  // /rules — тоже: на правило ссылается каждый бан, и прочитать его должен в
  // первую очередь тот, кого забанили, а он в кабинет уже не заходит.
  if (to.path.startsWith('/api') || to.path.startsWith('/oauth2')) return
  const publicPages = new Set(['/', '/login', '/setup', '/rules', '/servers'])
  if (publicPages.has(to.path)) return

  const token = useCookie<string | null>('noro_token')
  if (!token.value) {
    return navigateTo(`/login?next=${encodeURIComponent(to.fullPath)}`)
  }

  const auth = useAuth()
  const user = auth.user.value || await auth.loadMe()
  if (!user) {
    return navigateTo(`/login?next=${encodeURIComponent(to.fullPath)}`)
  }

  // Панель открывает любой узел `noro.admin.*`, а не только полный доступ:
  // модератор с одним правом на игроков должен попадать внутрь.
  if (to.path.startsWith('/admin') && !auth.canAdmin.value) {
    return navigateTo('/cabinet')
  }

  // Дальше — конкретная страница. Без права на неё человек летел бы в 403 уже
  // после загрузки, увидев пустой каркас и решив, что админка сломана.
  const required = adminPagePermission(to.path)
  if (required && !auth.hasPermission(required)) {
    return navigateTo('/admin')
  }
})

/** Право, без которого страницу админки открывать незачем. */
function adminPagePermission(path: string): string | null {
  const rules: [string, string][] = [
    ['/admin/users', 'noro.admin.users.view'],
    ['/admin/clients', 'noro.admin.servers.view'],
    ['/admin/mods', 'noro.admin.mods.view'],
    ['/admin/capes', 'noro.admin.capes.view'],
    ['/admin/roles', 'noro.admin.roles.view'],
    ['/admin/integrity', 'noro.admin.integrity.view'],
    ['/admin/blocklist', 'noro.admin.blocklist.view'],
    ['/admin/news', 'noro.admin.news.view'],
    ['/admin/rules', 'noro.admin.rules.view'],
    ['/admin/translations', 'noro.admin.translations.view'],
    ['/admin/wrapper', 'noro.admin.wrapper.view'],
    ['/admin/launcher/tokens', 'noro.admin.launcher.tokens'],
    ['/admin/launcher', 'noro.admin.launcher.view'],
    ['/admin/audit', 'noro.admin.audit'],
    ['/admin/support', 'noro.admin.support.logs'],
    ['/admin/apps', 'noro.admin.oauth.view'],
    ['/admin/settings', 'noro.admin.settings.view'],
  ]
  return rules.find(([prefix]) => path === prefix || path.startsWith(`${prefix}/`))?.[1] || null
}
