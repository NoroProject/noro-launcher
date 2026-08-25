/**
 * Ранжирование совпадений для спотлайта.
 *
 * Раньше был голый `includes`: «Резервная копия» и «Копия мастера» шли вперемешку
 * в порядке загрузки, а опечатка или сокращение не находили ничего. Точное
 * совпадение и начало слова человек имеет в виду почти всегда, поэтому они и
 * стоят выше простого вхождения.
 */

/** 0 — не совпало. Чем больше, тем ближе к тому, что искали. */
export function score(text: string, query: string): number {
    const s = text.toLowerCase()
    const q = query.toLowerCase()
    if (!q) return 1
    if (s === q) return 100
    if (s.startsWith(q)) return 80
    // Начало любого слова: «рез коп» должно находить «Резервная копия», а поиск
    // по пути /admin/backup — работать от «backup», а не только от «/admin».
    if (s.split(/[\s\-_/.:@]+/).some((w) => w.startsWith(q))) return 60
    if (s.includes(q)) return 40
    return subsequence(s, q) ? 20 : 0
}

/** Лучшее из нескольких полей: заголовок, подпись, путь. */
export function scoreAny(query: string, ...fields: (string | null | undefined)[]): number {
    let best = 0
    for (const f of fields) {
        if (f) best = Math.max(best, score(f, query))
    }
    return best
}

/**
 * Буквы запроса идут по порядку, но не подряд: «блкст» находит «блоклист».
 * Последняя линия — срабатывает, только когда всё остальное промахнулось.
 */
function subsequence(text: string, query: string): boolean {
    let i = 0
    for (const ch of text) {
        if (ch === query[i]) i++
        if (i === query.length) return true
    }
    return false
}
