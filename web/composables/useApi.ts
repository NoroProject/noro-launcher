type FetchOptions = Parameters<typeof $fetch>[1]

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

  const masterUrl = computed(() =>
    String(config.public.masterUrl || 'http://localhost:8080').replace(/\/$/, '')
  )
  const webUrl = computed(() =>
    String(config.public.webUrl || 'http://localhost:3000').replace(/\/$/, '')
  )

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
      await request('/auth/logout', { method: 'GET' })
    } catch {
      // Local cleanup still matters: the token may already be expired.
    }
    token.value = null
    refreshToken.value = null
  }

  function discordLoginUrl(redirectPath = '/login') {
    const redirect = import.meta.client
      ? `${window.location.origin}${redirectPath}`
      : `${webUrl.value}${redirectPath}`
    const url = new URL('/auth/discord/login', masterUrl.value)
    url.searchParams.set('redirect', redirect)
    return url.toString()
  }

  return {
    masterUrl,
    webUrl,
    token,
    refreshToken,
    request,
    upload,
    logout,
    discordLoginUrl
  }
}
