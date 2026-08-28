/** Rulebook: category tree, search and labels. Nuxt picks `utils/` up itself. */

import type { Rule, RuleCategory, RuleNode, RuleSanction } from '~/types/rules'

/** Passed in rather than imported: the catalog lives in the Nuxt context. */
type Translate = (key: string, args?: Record<string, string | number>) => string

export function kindLabel(kind: string, t: Translate): string {
  return t(`web-sanction-${kind.replace('_', '-')}`)
}

/**
 * Allowed sanction on one line: "Mute 30m–2h", "Ban from 7d". Durations stay
 * in the admin's own notation — spelling a range out in prose would have to be
 * redone for every language.
 */
export function formatSanction(sanction: RuleSanction, t: Translate): string {
  const kind = kindLabel(sanction.kind, t)
  if (sanction.kind === 'warn') return kind
  const { min_minutes: min, max_minutes: max } = sanction
  if (min && max) {
    return min === max
      ? t('web-sanction-exact', { kind, min: formatDuration(min) })
      : t('web-sanction-range', { kind, min: formatDuration(min), max: formatDuration(max) })
  }
  if (min) return t('web-sanction-from', { kind, min: formatDuration(min) })
  if (max) return t('web-sanction-upto', { kind, max: formatDuration(max) })
  return t('web-sanction-any', { kind })
}

/** Default duration for a variant: the lower bound, else the upper one. */
export function defaultMinutes(sanction: RuleSanction): number | null {
  if (sanction.kind === 'warn') return null
  return sanction.min_minutes ?? sanction.max_minutes ?? null
}

/** Whether a duration fits the variant. Mirrors the check on the server. */
export function sanctionAllows(sanction: RuleSanction, minutes: number | null): boolean {
  if (sanction.kind === 'warn') return true
  if (minutes === null) return sanction.max_minutes === null
  if (sanction.min_minutes !== null && minutes < sanction.min_minutes) return false
  if (sanction.max_minutes !== null && minutes > sanction.max_minutes) return false
  return true
}

/** Matches on code, title or body text. */
export function ruleMatches(rule: Rule, query: string): boolean {
  const q = query.trim().toLowerCase()
  if (!q) return true
  return (
    rule.code.toLowerCase().includes(q)
    || rule.title.toLowerCase().includes(q)
    || rule.description.toLowerCase().includes(q)
  )
}

/**
 * Flat lists into a tree of any depth.
 *
 * A category with no matching rules drops out entirely, so a search doesn't
 * leave a page of empty headings. A category whose parent never arrived
 * (deleted, or from another rulebook) becomes a root instead of vanishing
 * along with its rules.
 */
export function buildRuleTree(
  categories: RuleCategory[],
  rules: Rule[],
  query = '',
  keepEmpty = false,
): RuleNode[] {
  const matched = rules.filter(r => ruleMatches(r, query))
  const nodes = new Map<string, RuleNode>()
  for (const category of categories) {
    nodes.set(category.id, { category, rules: [], children: [], total: 0 })
  }
  for (const rule of matched) {
    const node = rule.category_id ? nodes.get(rule.category_id) : undefined
    if (node) node.rules.push(rule)
  }

  const roots: RuleNode[] = []
  for (const node of nodes.values()) {
    const parent = node.category.parent_id ? nodes.get(node.category.parent_id) : undefined
    if (parent && parent !== node) parent.children.push(node)
    else roots.push(node)
  }

  const sort = (list: RuleNode[]): RuleNode[] => {
    for (const node of list) {
      node.rules.sort(byOrder)
      node.children = sort(node.children)
      node.total = node.rules.length + node.children.reduce((sum, c) => sum + c.total, 0)
    }
    // The admin keeps empty categories: one was just created to be filled.
    return list
      .filter(node => keepEmpty || node.total > 0)
      .sort((a, b) => byOrder(a.category, b.category))
  }
  return sort(roots)
}

/** Rules with no category. Shown last, but not lost. */
export function looseRules(rules: Rule[], categories: RuleCategory[], query = ''): Rule[] {
  const known = new Set(categories.map(c => c.id))
  return rules
    .filter(r => (!r.category_id || !known.has(r.category_id)) && ruleMatches(r, query))
    .sort(byOrder)
}

/** Next free code in a category. The admin can edit it; usually they don't. */
export function nextRuleCode(categoryCode: string, siblings: Rule[]): string {
  const prefix = categoryCode.trim().replace(/\.$/, '')
  const used = siblings
    .map(r => Number(r.code.trim().split('.').pop()))
    .filter(n => Number.isInteger(n)) as number[]
  const next = used.length ? Math.max(...used) + 1 : 1
  return prefix ? `${prefix}.${next}` : String(next)
}

/** Next category number: "3" at the root, "2.4" inside the second category. */
export function nextCategoryCode(parentCode: string, siblings: RuleCategory[]): string {
  const prefix = parentCode.trim().replace(/\.$/, '')
  const used = siblings
    .map(c => Number(c.code.trim().split('.').pop()))
    .filter(n => Number.isInteger(n)) as number[]
  const next = used.length ? Math.max(...used) + 1 : 1
  return prefix ? `${prefix}.${next}` : String(next)
}

function byOrder(a: { sort_order: number, code: string }, b: { sort_order: number, code: string }) {
  return a.sort_order - b.sort_order || a.code.localeCompare(b.code, 'en', { numeric: true })
}
