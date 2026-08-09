/** Форматирование чисел и дат каталога. Nuxt подхватывает `utils/` сам. */

/** `1234567` → `1.2M`: в карточке важен порядок, а не точное число. */
export function compactNumber(value: number): string {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`;
  return String(value);
}

export function compactBytes(size: number): string {
  if (size >= 1024 * 1024) return `${(size / 1024 / 1024).toFixed(1)} MB`;
  if (size >= 1024) return `${Math.round(size / 1024)} KB`;
  return `${size} B`;
}

/** «3 дня назад» без библиотеки: точность до дня здесь и нужна. */
export function relativeDate(iso: string | null): string {
  if (!iso) return "";
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) return "";
  const days = Math.floor((Date.now() - then) / 86_400_000);
  if (days <= 0) return "today";
  if (days === 1) return "yesterday";
  if (days < 30) return `${days}d ago`;
  if (days < 365) return `${Math.floor(days / 30)}mo ago`;
  return `${Math.floor(days / 365)}y ago`;
}

/** Цвет канала версии: релиз — спокойный, альфа — тревожный. */
export function channelColor(channel: string): string {
  if (channel === "beta") return "var(--noro-amber)";
  if (channel === "alpha") return "var(--noro-danger)";
  return "var(--noro-green)";
}
