/**
 * Чем можно войти на этом инстансе.
 *
 * Список приходит с мастера, а не зашит в вёрстку: платформы включает и
 * выключает оператор в админке, и кнопка, за которой ничего нет, — прямой путь
 * в непонятный отказ.
 */

export interface AuthProviderInfo {
  provider: string
  name: string
}

export interface AuthMethods {
  providers: AuthProviderInfo[]
  /** Вход по passkey: включён оператором и технически возможен. */
  passkey: boolean
}

export async function loadAuthMethods(): Promise<AuthMethods> {
  const api = useApi()
  try {
    return await api.request<AuthMethods>('/api/auth/methods')
  } catch {
    // Мастер недоступен — страница входа всё равно должна открыться и
    // показать внятную ошибку, а не белый экран.
    return { providers: [], passkey: false }
  }
}

/** Иллюстрация страницы входа. Пусто — там своя заглушка. */
export async function loadLoginImage(): Promise<string> {
  const api = useApi()
  try {
    const res = await api.request<{ login_image_url?: string }>('/api/public/settings')
    return res.login_image_url || ''
  } catch {
    return ''
  }
}
