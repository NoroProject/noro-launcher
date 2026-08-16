export default defineNuxtRouteMiddleware(async (to) => {
  // /setup доступен без входа: аккаунтов на ненастроенном инстансе ещё нет.
  const publicPages = new Set(['/', '/login', '/setup'])
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

  if (to.path.startsWith('/admin') && !auth.hasPermission('noro.admin.*')) {
    return navigateTo('/cabinet')
  }
})
