/**
 * Локализация сайта поверх тех же Fluent-каталогов, что и у лаунчера.
 *
 * Каталог берётся с мастера: встроенный текст плюс правки из админки. Порядок
 * поиска ключа — правки → встроенный → английский → сам ключ, как в
 * `crates/i18n`: недопереведённая страница должна остаться читаемой.
 */

import { FluentBundle, FluentResource } from '@fluent/bundle'

/** Ответ GET /api/launcher/locales/{locale}. */
interface Catalog {
  locale: string
  sha1: string
  /** Правки из админки. Пусто — используется только встроенный. */
  ftl: string
  builtin: string
}

export type TranslateArgs = Record<string, string | number>

function bundleOf(locale: string, ftl: string) {
  const bundle = new FluentBundle(locale, { useIsolating: false })
  // Fluent возвращает всё, что успел разобрать: строки до ошибки рабочие,
  // и терять из-за одной опечатки в админке весь каталог незачем.
  bundle.addResource(new FluentResource(ftl))
  return bundle
}

/** Каталоги в порядке поиска: активный язык, затем английский как запас. */
export function buildBundles(active: Catalog, fallback: Catalog | null) {
  const sources = [active, fallback].filter(Boolean) as Catalog[]
  return sources.flatMap(c => [c.ftl, c.builtin].filter(Boolean).map(ftl => bundleOf(c.locale, ftl)))
}

export function translate(bundles: FluentBundle[], key: string, args?: TranslateArgs) {
  for (const bundle of bundles) {
    const message = bundle.getMessage(key)
    if (!message?.value) continue
    return bundle.formatPattern(message.value, args)
  }
  // Ключ виден в интерфейсе как есть: так сразу заметно, чего нет в каталоге.
  return key
}

export async function fetchCatalog(locale: string) {
  const { request } = useApi()
  return await request<Catalog>(`/api/launcher/locales/${locale}`)
}
