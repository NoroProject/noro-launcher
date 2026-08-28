/** Parsing punishment durations. Nuxt picks `utils/` up on its own. */

/** Minutes per unit. A month is 30 days and a year 365 — close enough for a ban. */
const UNITS: Record<string, number> = {
  m: 1,
  min: 1,
  h: 60,
  hour: 60,
  d: 1440,
  day: 1440,
  w: 10080,
  week: 10080,
  mo: 43200,
  y: 525600,
};

/** Order matters: `mo` and `min` have to be tried before `m`. */
const TOKEN = /(\d+)\s*(mo|min|month|hour|week|day|m|h|d|w|y)s?/g;

/**
 * `7d`, `12h`, `1d 6h`, `30m`, `2w`, `3mo` → minutes.
 *
 * `null` is forever (empty string). `NaN` means it didn't parse — the field
 * shows an error rather than sending the server a duration nobody meant.
 *
 * A bare number is hours: the field was labelled "Hours" for years.
 */
export function parseDuration(input: string): number | null {
  const text = input.trim().toLowerCase();
  if (!text || text === "forever" || text === "perm") return null;
  // Zero is neither forever nor a minute — more likely a half-typed duration.
  if (/^\d+$/.test(text)) return Number(text) ? Number(text) * 60 : NaN;

  let total = 0;
  const rest = text.replace(TOKEN, (_, amount: string, unit: string) => {
    total += Number(amount) * UNITS[unit]!;
    return "";
  });
  // Leftovers like `7x` are a typo, not a duration.
  if (rest.replace(/[\s,+и]/g, "") || !total) return NaN;
  return total;
}

/** `10830` → `7d 12h 30m`, so the admin can see the duration was read right. */
export function formatDuration(minutes: number): string {
  const parts: string[] = [];
  let left = minutes;
  for (const [unit, size] of [
    // Years and months belong here, or a year-long ban reads as `52w 1d`.
    // Weeks are left out on purpose: a typed `7d` shouldn't come back as `1w`.
    ["y", 525600],
    ["mo", 43200],
    ["d", 1440],
    ["h", 60],
    ["m", 1],
  ] as const) {
    const count = Math.floor(left / size);
    if (count) parts.push(`${count}${unit}`);
    left -= count * size;
  }
  return parts.join(" ");
}
