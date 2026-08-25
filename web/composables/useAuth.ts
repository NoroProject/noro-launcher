import type { UserProfile } from '~/types/api'

export function useAuth() {
  const api = useApi()
  const user = useState<UserProfile | null>('noro_user', () => null)
  const loading = useState<boolean>('noro_auth_loading', () => false)
  const error = useState<string | null>('noro_auth_error', () => null)

  const loggedIn = computed(() => Boolean(api.token.value))

  async function loadMe() {
    if (!api.token.value) {
      user.value = null
      return null
    }
    loading.value = true
    error.value = null
    try {
      user.value = await api.request<UserProfile>('/api/me')
      return user.value
    } catch (e) {
      error.value = humanError(e)
      user.value = null
      api.token.value = null
      return null
    } finally {
      loading.value = false
    }
  }

  async function signOut() {
    await api.logout()
    user.value = null
    await navigateTo('/login')
  }

  /** Все права: свои и пришедшие от ролей. */
  const permissions = computed(() => {
    if (!user.value) return [] as string[]
    return [...user.value.permissions, ...user.value.roles.flatMap(role => role.permissions)]
  })

  function hasPermission(permission: string) {
    return permissions.value.some(perm => permissionMatches(perm, permission))
  }

  /** Хотя бы одно из перечисленных прав: блок панели обычно открывает любое. */
  function hasAny(...list: string[]) {
    return list.some(hasPermission)
  }

  /**
   * Отдельного права «войти в админку» нет: панель открывает любой узел
   * `noro.admin.*`. Иначе выдача точечного права оставляла бы человека перед
   * закрытой дверью, и это выглядело бы как поломка, а не как настройка.
   */
  const canAdmin = computed(() =>
    permissions.value.some(
      perm => perm === '*' || perm.startsWith('noro.admin.') || perm.startsWith('noro.mod.'),
    ),
  )

  return {
    ...api,
    user,
    loading,
    error,
    loggedIn,
    loadMe,
    signOut,
    permissions,
    hasPermission,
    hasAny,
    canAdmin
  }
}

export function permissionMatches(pattern: string, target: string) {
  if (pattern === '*' || pattern === target) return true
  if (pattern.endsWith('.*')) {
    const prefix = pattern.slice(0, -2)
    if (target === prefix || target.startsWith(`${prefix}.`)) return true
  }
  return false
}

export function humanError(error: unknown) {
  // Разбор конверта — в `utils/api-error.ts`: он же нужен уведомлениям, и две
  // копии успели разойтись в том, какие поля вообще смотрят.
  const message = apiErrorLabel(error)
  if (message) return message
  if (error instanceof Error) return error.message
  return 'Request failed'
}
