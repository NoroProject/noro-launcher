/**
 * Ranking matches for spotlight. An exact hit or a word start is almost always
 * what was meant, so both outrank a bare substring.
 */

/** 0 means no match. Higher is closer to what was typed. */
export function score(text: string, query: string): number {
    const s = text.toLowerCase()
    const q = query.toLowerCase()
    if (!q) return 1
    if (s === q) return 100
    if (s.startsWith(q)) return 80
    // Start of any word, so searching /admin/backup works from "backup" and
    // not only from "/admin".
    if (s.split(/[\s\-_/.:@]+/).some((w) => w.startsWith(q))) return 60
    if (s.includes(q)) return 40
    return subsequence(s, q) ? 20 : 0
}

/** Best score across several fields: title, subtitle, path. */
export function scoreAny(query: string, ...fields: (string | null | undefined)[]): number {
    let best = 0
    for (const f of fields) {
        if (f) best = Math.max(best, score(f, query))
    }
    return best
}

/**
 * Query letters in order but not adjacent: "blklst" finds "blocklist". Last
 * resort — only reached once everything else has missed.
 */
function subsequence(text: string, query: string): boolean {
    let i = 0
    for (const ch of text) {
        if (ch === query[i]) i++
        if (i === query.length) return true
    }
    return false
}
