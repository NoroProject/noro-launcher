/** Number and date formatting for the catalog. Nuxt picks `utils/` up itself. */

/** `1234567` → `1.2M`. A card wants the order of magnitude, not the number. */
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

/** "3 days ago" without a library — day precision is all this needs. */
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
 * Same thing through the catalog. `relativeDate` hardcodes English and slips
 * past translation, so a localised page ends up with "9d ago" in it.
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
 * Hours and minutes. In a case timeline "today" says nothing — events land
 * minutes apart, and only the clock shows the order and the gaps.
 */
export function clockTime(iso: string): string {
  const at = new Date(iso);
  if (Number.isNaN(at.getTime())) return "";
  return at.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit", hour12: false });
}

/** Date of an event, for when a case runs past midnight. */
export function shortDate(iso: string): string {
  const at = new Date(iso);
  if (Number.isNaN(at.getTime())) return "";
  return at.toLocaleDateString(undefined, { day: "2-digit", month: "short" });
}

/** Human-facing case number: `N-000000001`. Nine digits will outlast us. */
export function caseNumber(value: number): string {
  return `N-${String(value).padStart(9, "0")}`;
}

/** Release channel colour: calm for stable, alarming for alpha. */
export function channelColor(channel: string): string {
  if (channel === "beta") return "var(--noro-amber)";
  if (channel === "alpha") return "var(--noro-danger)";
  return "var(--noro-green)";
}
