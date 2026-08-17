/**
 * Черновик многоязычного текста для модалок свода.
 *
 * Мастер хранит исходный текст в самой записи, а переводы — отдельной
 * таблицей. Редактору удобнее один список «язык → текст», поэтому здесь он
 * собирается на входе и разбирается обратно на выходе.
 */

import type { LocalizedText } from '~/types/rules'

/** Строка перевода с мастера: у правила поле `title`, у раздела — `name`. */
interface TranslationRow {
  locale: string
  title?: string
  name?: string
  description: string
}

export function useLocalizedDraft() {
  const auth = useAuth()
  const items = ref<LocalizedText[]>([])

  const base = computed(
    () => items.value.find(i => i.locale === BASE_LOCALE) || { title: '', description: '' },
  )

  /**
   * Заполнить вкладки. `path` не задан у новой записи — переводить ещё нечего.
   * Сбой загрузки оставляет вкладки пустыми: сохранить правило важнее, чем
   * показать его переводы, а стереть их пустой формой мешает проверка в submit.
   */
  async function load(source: { title: string, description: string }, path?: string) {
    items.value = LOCALES.map(l =>
      l.code === BASE_LOCALE
        ? { locale: l.code, title: source.title, description: source.description }
        : { locale: l.code, title: '', description: '' },
    )
    if (!path) return
    let rows: TranslationRow[] = []
    try {
      rows = await auth.request<TranslationRow[]>(path)
    } catch (e) {
      console.error(e)
      return
    }
    items.value = items.value.map((item) => {
      const row = rows.find(r => r.locale === item.locale)
      if (!row || item.locale === BASE_LOCALE) return item
      return { ...item, title: row.title ?? row.name ?? '', description: row.description }
    })
  }

  /** Что уходит в поле `translations`: базовый язык — не перевод. */
  function payload() {
    return items.value
      .filter(i => i.locale !== BASE_LOCALE && i.title.trim())
      .map(i => ({ locale: i.locale, title: i.title.trim(), description: i.description.trim() }))
  }

  return { items, base, load, payload }
}
