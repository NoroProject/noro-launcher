/**
 * Каталог локализации загружается до первой отрисовки.
 *
 * Иначе страница успела бы моргнуть ключами: `t()` должен отвечать синхронно
 * из любого компонента, а каталог живёт на мастере.
 */

import type { FluentBundle } from '@fluent/bundle'

export default defineNuxtPlugin(async () => {
  const choice = useCookie<string | null>('noro_locale', {
    maxAge: 60 * 60 * 24 * 365,
    sameSite: 'lax',
  })
  // Каталоги переживают гидратацию: то, что приехало в payload с сервера,
  // клиент не запрашивает заново.
  const cache = useState<Record<string, Awaited<ReturnType<typeof fetchCatalog>>>>(
    'i18n-catalogs',
    () => ({}),
  )
  const locale = useState('i18n-locale', () => choice.value || BASE_LOCALE)
  const bundles = shallowRef<FluentBundle[]>([])

  async function ensure(code: string) {
    const known = cache.value[code]
    if (known) return known
    const catalog = await fetchCatalog(code)
    cache.value[code] = catalog
    return catalog
  }

  async function apply(code: string) {
    const active = await ensure(code)
    const fallback = code === BASE_LOCALE ? null : await ensure(BASE_LOCALE)
    bundles.value = buildBundles(active, fallback)
    locale.value = code
  }

  async function setLocale(code: string) {
    if (code === locale.value) return
    choice.value = code
    await apply(code)
  }

  try {
    await apply(locale.value)
  } catch (e) {
    // Без каталога страница покажет сами ключи. Это заметно, но лучше, чем
    // пустой сайт из-за недоступного мастера.
    console.error('locale catalog unavailable', e)
  }

  return {
    provide: {
      i18n: {
        locale: readonly(locale),
        setLocale,
        t: (key: string, args?: TranslateArgs) => translate(bundles.value, key, args),
      },
    },
  }
})
