type FetchOptions = Parameters<typeof $fetch>[1]

/**
 * A URL that must come from the environment.
 *
 * In dev an unset variable falls back to the local master/site, which is where
 * a dev server actually runs. In a real build it throws: guessing a domain here
 * meant every deployment without the variable silently talked to ours.
 */
function configured(value: unknown, name: string) {
  const url = String(value || '').replace(/\/$/, '')
  if (url) return url
  if (import.meta.dev) {
    return name === 'NUXT_PUBLIC_WEB_URL' ? 'http://localhost:3000' : 'http://localhost:8080'
  }
  throw new Error(`${name} is not set — this build has no server to talk to.`)
}

export function useApi() {
  const config = useRuntimeConfig()
  const token = useCookie<string | null>('noro_token', {
    sameSite: 'lax',
    secure: import.meta.client && window.location.protocol === 'https:'
  })
  const refreshToken = useCookie<string | null>('noro_refresh', {
    sameSite: 'lax',
    secure: import.meta.client && window.location.protocol === 'https:'
  })

  const masterUrl = computed(() => configured(config.public.masterUrl, 'NUXT_PUBLIC_MASTER_URL'))
  const webUrl = computed(() => configured(config.public.webUrl, 'NUXT_PUBLIC_WEB_URL'))

  async function request<T>(path: string, options: FetchOptions = {}) {
    const headers = new Headers(options.headers as HeadersInit | undefined)
    if (token.value) {
      headers.set('Authorization', `Bearer ${token.value}`)
    }
    return await $fetch<T>(path, {
      baseURL: masterUrl.value,
      ...options,
      headers
    })
  }

  /**
   * Списочная ручка мастера отдаёт `{ items, total }`, а вызывающему обычно
   * нужен только массив.
   *
   * Отдельным методом, а не разворотом на каждом месте вызова: таких мест
   * больше двадцати, и стоит забыть одно — страница молча покажет пустой
   * список вместо ошибки. Счётчик берут те, кому он нужен, через `request`.
   */
  async function requestList<T>(path: string, options: FetchOptions = {}) {
    const page = await request<{ items: T[]; total: number }>(path, options)
    return page?.items ?? []
  }

  async function upload<T>(
    path: string,
    field: string,
    file: File,
    extra?: Record<string, string>,
    method: 'POST' | 'PUT' = 'POST'
  ) {
    const body = new FormData()
    for (const [key, value] of Object.entries(extra || {})) {
      body.append(key, value)
    }
    body.append(field, file)
    return await request<T>(path, { method, body })
  }

  async function logout() {
    try {
      await request('/auth/logout', { method: 'POST' })
    } catch {
      // Local cleanup still matters: the token may already be expired.
    }
    token.value = null
    refreshToken.value = null
  }

  /** Ссылка входа через платформу: discord, twitch, google. */
  function providerLoginUrl(provider: string, redirectPath = '/login') {
    const redirect = import.meta.client
      ? `${window.location.origin}${redirectPath}`
      : `${webUrl.value}${redirectPath}`
    const url = new URL(`/auth/${provider}/login`, masterUrl.value)
    url.searchParams.set('redirect', redirect)
    return url.toString()
  }

  return {
    masterUrl,
    webUrl,
    token,
    refreshToken,
    request,
    requestList,
    upload,
    logout,
    providerLoginUrl
  }
}
