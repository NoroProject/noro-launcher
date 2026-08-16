import type { SetupStatus, SigningKey } from '~/types/setup'

/**
 * Клиент визарда первичной настройки.
 *
 * Токен держится в sessionStorage, а не в cookie: он одноразовый и живёт
 * ровно одну установку. Класть его рядом с обычной сессией — значит забыть
 * потом убрать.
 */
export function useSetup() {
  const api = useApi()
  const TOKEN_KEY = 'noro_setup_token'

  const token = ref<string>('')
  const status = ref<SetupStatus | null>(null)
  const error = ref<string | null>(null)

  if (import.meta.client) {
    token.value = sessionStorage.getItem(TOKEN_KEY) || ''
  }

  function rememberToken(value: string) {
    token.value = value.trim()
    if (import.meta.client) sessionStorage.setItem(TOKEN_KEY, token.value)
  }

  function forgetToken() {
    token.value = ''
    if (import.meta.client) sessionStorage.removeItem(TOKEN_KEY)
  }

  function authed<T>(path: string, options: Record<string, unknown> = {}) {
    return api.request<T>(path, {
      ...options,
      headers: { Authorization: `Bearer ${token.value}` },
    } as never)
  }

  async function loadStatus() {
    status.value = await api.request<SetupStatus>('/api/setup/status')
    return status.value
  }

  const saveSettings = (settings: Record<string, string>) =>
    authed<{ ok: boolean; saved: number }>('/api/setup/settings', {
      method: 'POST',
      body: { settings },
    })

  const generateSigningKey = () =>
    authed<SigningKey>('/api/setup/signing-key', { method: 'POST' })

  const envBlock = () => authed<{ env: string }>('/api/setup/env')

  const complete = () =>
    authed<{ ok: boolean; restart_required: boolean }>('/api/setup/complete', {
      method: 'POST',
    })

  return {
    token,
    status,
    error,
    rememberToken,
    forgetToken,
    loadStatus,
    saveSettings,
    generateSigningKey,
    envBlock,
    complete,
  }
}
