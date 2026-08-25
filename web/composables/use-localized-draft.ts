/**
 * Черновик многоязычного текста для модалок свода.
 */

import type { LocalizedText } from '~/types/rules'

interface TranslationRow {
  locale: string
  title?: string
  name?: string
  description: string
  punish_reason?: string
}

export function useLocalizedDraft() {
  const auth = useAuth()
  const items = ref<LocalizedText[]>([])

  const base = computed(
    () => items.value.find(i => i.locale === BASE_LOCALE) || { title: '', description: '', punish_reason: '' },
  )

  async function load(source: { title: string, description: string, punish_reason?: string }, path?: string) {
    let availableCodes = ['ru', 'en']
    try {
      const localesList = await auth.request<{ locale: string }[]>('/api/launcher/locales')
      if (localesList && localesList.length) {
        availableCodes = localesList.map(l => l.locale)
      }
    } catch {
      // fallback
    }

    let rows: TranslationRow[] = []
    if (path) {
      try {
        rows = await auth.requestList<TranslationRow>(path)
      } catch (e) {
        console.error(e)
      }
    }

    const allCodes = new Set([BASE_LOCALE, ...availableCodes, ...rows.map(r => r.locale)])
    items.value = Array.from(allCodes).map(code => {
      if (code === BASE_LOCALE) {
        return {
          locale: code,
          title: source.title,
          description: source.description,
          punish_reason: source.punish_reason ?? '',
        }
      }
      const row = rows.find(r => r.locale === code)
      return {
        locale: code,
        title: row?.title ?? row?.name ?? '',
        description: row?.description ?? '',
        punish_reason: row?.punish_reason ?? '',
      }
    })
  }

  function payload() {
    return items.value
      .filter(i => i.locale !== BASE_LOCALE && i.title.trim())
      .map(i => ({
        locale: i.locale,
        title: i.title.trim(),
        description: i.description.trim(),
        punish_reason: (i.punish_reason ?? '').trim(),
      }))
  }

  return { items, base, load, payload }
}
