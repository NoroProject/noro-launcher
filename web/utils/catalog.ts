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

/**
 * «3 дня назад» словами каталога.
 *
 * `relativeDate` отдаёт английские строки мимо переводов: в русской админке
 * рядом с «открыто» появлялось «9d ago». Здесь то же самое, но ключами.
 */
export function relativeDateT(iso: string | null, t: (key: string, args?: Record<string, string | number>) => string): string {
  if (!iso) return "";
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) return "";
  const days = Math.floor((Date.now() - then) / 86_400_000);
  if (days <= 0) return t("web-date-today");
  if (days === 1) return t("web-date-yesterday");
  if (days < 30) return t("web-date-days", { count: days });
  if (days < 365) return t("web-date-months", { count: Math.floor(days / 30) });
  return t("web-date-years", { count: Math.floor(days / 365) });
}

/**
 * Часы и минуты события.
 *
 * В ленте разбора «today» не значит ничего: события идут минутами друг за
 * другом, и порядок с интервалом видно только по времени.
 */
export function clockTime(iso: string): string {
  const at = new Date(iso);
  if (Number.isNaN(at.getTime())) return "";
  return at.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit", hour12: false });
}

/** Дата события, когда разбор перевалил за полночь. */
export function shortDate(iso: string): string {
  const at = new Date(iso);
  if (Number.isNaN(at.getTime())) return "";
  return at.toLocaleDateString(undefined, { day: "2-digit", month: "short" });
}

/** Номер дела для человека: `N-000000001`. Девять знаков хватит навсегда. */
export function caseNumber(value: number): string {
  return `N-${String(value).padStart(9, "0")}`;
}

/** Цвет канала версии: релиз — спокойный, альфа — тревожный. */
export function channelColor(channel: string): string {
  if (channel === "beta") return "var(--noro-amber)";
  if (channel === "alpha") return "var(--noro-danger)";
  return "var(--noro-green)";
}
