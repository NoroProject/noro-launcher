/** Разбор срока наказания. Nuxt подхватывает `utils/` сам. */

/** Сколько минут в единице. Месяц — 30 дней, год — 365: сроку бана точнее не надо. */
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

/** Порядок важен: `mo` и `min` должны примеряться раньше, чем `m`. */
const TOKEN = /(\d+)\s*(mo|min|month|hour|week|day|m|h|d|w|y)s?/g;

/**
 * `7d`, `12h`, `1d 6h`, `30m`, `2w`, `3mo` → минуты.
 *
 * `null` — навсегда (пустая строка). `NaN` — не разобрано: поле подсветит
 * ошибку, а не отправит на сервер срок, который админ не имел в виду.
 *
 * Голое число — часы: поле годами было «Hours», и переучивать некого.
 */
export function parseDuration(input: string): number | null {
  const text = input.trim().toLowerCase();
  if (!text || text === "forever" || text === "perm") return null;
  // Ноль — не «навсегда» и не минута: скорее всего срок недонабрали.
  if (/^\d+$/.test(text)) return Number(text) ? Number(text) * 60 : NaN;

  let total = 0;
  const rest = text.replace(TOKEN, (_, amount: string, unit: string) => {
    total += Number(amount) * UNITS[unit]!;
    return "";
  });
  // Хвост вроде `7x` или `5 бан` — это опечатка, а не срок.
  if (rest.replace(/[\s,+и]/g, "") || !total) return NaN;
  return total;
}

/** `10830` → `7d 12h 30m`: подтверждение, что срок понят так же, как задуман. */
export function formatDuration(minutes: number): string {
  const parts: string[] = [];
  let left = minutes;
  for (const [unit, size] of [
    // Год и месяц — здесь же, иначе годовой бан читался бы как `52w 1d`.
    // Недель нет намеренно: набранное `7d` не должно превращаться в `1w`.
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
