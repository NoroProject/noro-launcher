/**
 * Брендинг инстанса: название, логотип и иллюстрации.
 *
 * Один `useAsyncData` на всё приложение — ключ общий, поэтому шапка, главная и
 * страница входа обходятся одним запросом к мастеру. Раньше каждая страница
 * ходила за настройками сама, и главная делала это относительным `useFetch`,
 * который уходил в 404 сайта вместо мастера.
 */

export interface PublicSettings {
  instance_name: string
  /** Пусто — сайт рисует значок из сборки. */
  logo_url: string
  hero_image_url: string
  /** В иллюстрации есть прозрачные места: подложка с рамкой ей не нужна. */
  hero_image_transparent: boolean
  login_image_url: string
}

const FALLBACK: PublicSettings = {
  instance_name: 'Noro Launcher',
  logo_url: '',
  hero_image_url: '',
  hero_image_transparent: false,
  login_image_url: '',
}

export function usePublicSettings() {
  const api = useApi()

  const { data } = useAsyncData('public-settings', async () => {
    try {
      return { ...FALLBACK, ...(await api.request<Partial<PublicSettings>>('/api/public/settings')) }
    } catch {
      // Мастер молчит — страница остаётся рабочей на своих запасных картинках.
      return FALLBACK
    }
  }, { default: () => FALLBACK })

  return computed(() => data.value ?? FALLBACK)
}
