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

  function hasPermission(permission: string) {
    if (!user.value) return false
    if (user.value.permissions.includes('*')) return true
    const direct = user.value.permissions
    const rolePerms = user.value.roles.flatMap(role => role.permissions)
    return [...direct, ...rolePerms].some(perm => permissionMatches(perm, permission))
  }

  return {
    ...api,
    user,
    loading,
    error,
    loggedIn,
    loadMe,
    signOut,
    hasPermission
  }
}

export function permissionMatches(pattern: string, target: string) {
  if (pattern === '*' || pattern === target) return true
  if (pattern.endsWith('.*')) {
    const prefix = pattern.slice(0, -2)
    return target === prefix || target.startsWith(`${prefix}.`)
  }
  return false
}

export function humanError(error: unknown) {
  if (typeof error === 'object' && error && 'data' in error) {
    const data = (error as { data?: unknown }).data
    if (typeof data === 'string') return data
    if (typeof data === 'object' && data && 'message' in data) {
      return String((data as { message?: unknown }).message)
    }
  }
  if (error instanceof Error) return error.message
  return 'Request failed'
}
