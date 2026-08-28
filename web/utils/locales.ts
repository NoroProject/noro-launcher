/** Language catalog and formatting helpers. */

export interface LocaleOption {
  code: string
  label: string
  short: string
}

export const KNOWN_LOCALES: Record<string, { label: string, short: string }> = {
  ru: { label: 'Русский', short: 'RU' },
  en: { label: 'English', short: 'ENG' },
  es: { label: 'Español', short: 'ESP' },
  de: { label: 'Deutsch', short: 'GER' },
  fr: { label: 'Français', short: 'FRA' },
  zh: { label: '中文', short: 'ZH' },
  ja: { label: '日本語', short: 'JA' },
  uk: { label: 'Українська', short: 'UKR' },
  'pt-br': { label: 'Português (Brasil)', short: 'BR' },
  pt: { label: 'Português', short: 'PT' },
  it: { label: 'Italiano', short: 'ITA' },
  pl: { label: 'Polski', short: 'POL' },
  tr: { label: 'Türkçe', short: 'TUR' },
}

export const LOCALES: LocaleOption[] = [
  { code: 'ru', label: 'Русский', short: 'RU' },
  { code: 'en', label: 'English', short: 'ENG' },
]

export const BASE_LOCALE = 'ru'

export function getLocaleOption(code: string): LocaleOption {
  const known = KNOWN_LOCALES[code.toLowerCase()]
  if (known) {
    return { code, ...known }
  }
  return {
    code,
    label: code.toUpperCase(),
    short: code.substring(0, 3).toUpperCase(),
  }
}

export function localeLabel(code: string) {
  return getLocaleOption(code).label
}
