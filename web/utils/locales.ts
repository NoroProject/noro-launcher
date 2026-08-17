/** Языки сайта. Тот же список, что у лаунчера: каталог один на оба. */

export interface LocaleOption {
  code: string
  label: string
  /** Короткая подпись для переключателя в шапке. */
  short: string
}

export const LOCALES: LocaleOption[] = [
  { code: 'ru', label: 'Русский', short: 'RU' },
  { code: 'en', label: 'English', short: 'ENG' },
]

/**
 * Язык, на котором написан исходный текст.
 *
 * Для содержимого свода это не «ещё один перевод», а сама запись: её текст
 * лежит в `rules.title`, и с него читают все языки, на которые правило не
 * перевели.
 */
export const BASE_LOCALE = 'ru'

export function localeLabel(code: string) {
  return LOCALES.find(l => l.code === code)?.label || code
}
