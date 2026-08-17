/** Свод правил: дерево разделов, поиск и подписи. Nuxt подхватывает `utils/`. */

import type { Rule, RuleCategory, RuleNode, RuleSanction } from '~/types/rules'

/** Перевод: сюда его передают, потому что каталог живёт в контексте Nuxt. */
type Translate = (key: string, args?: Record<string, string | number>) => string

/** Название вида наказания: `server_ban` → «Бан на сервере». */
export function kindLabel(kind: string, t: Translate): string {
  return t(`web-sanction-${kind.replace('_', '-')}`)
}

/**
 * Допустимое наказание одной строкой: «Мут 30m–2h», «Бан от 7d». Сроки
 * остаются в записи админа: переводить вилку в прозу («не более двух часов»)
 * пришлось бы на каждом языке заново.
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

/** Срок по умолчанию для варианта: нижняя граница, иначе верхняя. */
export function defaultMinutes(sanction: RuleSanction): number | null {
  if (sanction.kind === 'warn') return null
  return sanction.min_minutes ?? sanction.max_minutes ?? null
}

/** Укладывается ли срок в рамки варианта. Та же проверка, что и на сервере. */
export function sanctionAllows(sanction: RuleSanction, minutes: number | null): boolean {
  if (sanction.kind === 'warn') return true
  if (minutes === null) return sanction.max_minutes === null
  if (sanction.min_minutes !== null && minutes < sanction.min_minutes) return false
  if (sanction.max_minutes !== null && minutes > sanction.max_minutes) return false
  return true
}

/** Совпадает ли правило с поисковым запросом: код, заголовок или текст. */
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
 * Плоские списки — в дерево любой глубины.
 *
 * Раздел без единого подходящего правила выпадает целиком: при поиске «мут»
 * страница, состоящая из пустых заголовков, ничего не отвечает на вопрос.
 * Раздел, чей родитель не пришёл (удалён, чужой свод), становится корневым —
 * иначе его правила исчезли бы вместе с ним.
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
    // В админке пустой раздел остаётся: его только что завели, чтобы наполнить.
    return list
      .filter(node => keepEmpty || node.total > 0)
      .sort((a, b) => byOrder(a.category, b.category))
  }
  return sort(roots)
}

/** Правила вне разделов: показываются последними, но не теряются. */
export function looseRules(rules: Rule[], categories: RuleCategory[], query = ''): Rule[] {
  const known = new Set(categories.map(c => c.id))
  return rules
    .filter(r => (!r.category_id || !known.has(r.category_id)) && ruleMatches(r, query))
    .sort(byOrder)
}

/**
 * Следующий свободный код в разделе: «1.» + номер. Админ его правит, но в
 * девяти случаях из десяти правило просто дописывают в конец раздела.
 */
export function nextRuleCode(categoryCode: string, siblings: Rule[]): string {
  const prefix = categoryCode.trim().replace(/\.$/, '')
  const used = siblings
    .map(r => Number(r.code.trim().split('.').pop()))
    .filter(n => Number.isInteger(n)) as number[]
  const next = used.length ? Math.max(...used) + 1 : 1
  return prefix ? `${prefix}.${next}` : String(next)
}

/** Следующий номер раздела: «3» в корне, «2.4» внутри второго раздела. */
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
