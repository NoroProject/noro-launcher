/** Админский свод правил: загрузка, сохранение и порядок. */

import type { Rule, RuleCategory, RuleSanction } from '~/types/rules'
import type { ServerRow } from '~/types/api'

export function useAdminRules() {
  const auth = useAuth()
  const notify = useNotify()

  const categories = ref<RuleCategory[]>([])
  const rules = ref<Rule[]>([])
  const sanctions = ref<RuleSanction[]>([])
  const servers = ref<ServerRow[]>([])
  const pending = ref(false)
  /** Фильтр по своду: `''` — показать всё сразу. */
  const scope = ref<string>('')
  const search = ref('')

  async function load() {
    pending.value = true
    try {
      const [c, r, s] = await Promise.all([
        auth.requestList<RuleCategory>('/api/admin/rules/categories'),
        auth.request<{ rules: Rule[], sanctions: RuleSanction[] }>('/api/admin/rules'),
        auth.requestList<ServerRow>('/api/admin/servers'),
      ])
      categories.value = c
      rules.value = r.rules
      sanctions.value = r.sanctions
      servers.value = s
    } catch (e) {
      notify.fail(e, 'Failed to load rules')
    } finally {
      pending.value = false
    }
  }

  const inScope = <T extends { server_id: string | null }>(items: T[]) =>
    scope.value === '' ? items : items.filter(i => (i.server_id || '') === scope.value)

  const tree = computed(() =>
    buildRuleTree(inScope(categories.value), inScope(rules.value), search.value, !search.value),
  )
  const loose = computed(() =>
    looseRules(inScope(rules.value), inScope(categories.value), search.value),
  )

  /** Санкции одного правила: дерево и модалка берут их отсюда. */
  function sanctionsOf(ruleId: string) {
    return sanctions.value.filter(s => s.rule_id === ruleId)
  }

  function serverName(id: string | null) {
    if (!id) return 'General'
    return servers.value.find(s => s.id === id)?.name || id.slice(0, 8)
  }

  async function save(path: string, body: Record<string, unknown>, id?: string) {
    await auth.request(id ? `${path}/${id}` : path, { method: id ? 'PUT' : 'POST', body })
    await load()
    notify.ok(id ? 'Updated' : 'Created')
  }

  async function remove(path: string, id: string) {
    await auth.request(`${path}/${id}`, { method: 'DELETE' })
    await load()
    notify.ok('Deleted')
  }

  /**
   * Порядок меняется обменом мест в общем списке: у элементов один сквозной
   * `sort_order`, поэтому переставить только соседей нельзя — их номера
   * налезли бы на чужие.
   */
  async function move(kind: 'rules' | 'categories', id: string, delta: number) {
    const list = kind === 'rules'
      ? [...rules.value].sort((a, b) => a.sort_order - b.sort_order)
      : [...categories.value].sort((a, b) => a.sort_order - b.sort_order)
    const from = list.findIndex(i => i.id === id)
    const to = from + delta
    if (from < 0 || to < 0 || to >= list.length) return
    const order = list.map(i => i.id);
    [order[from], order[to]] = [order[to]!, order[from]!]
    const path = kind === 'rules' ? '/api/admin/rules/reorder' : '/api/admin/rules/categories/reorder'
    try {
      await auth.request(path, { method: 'PUT', body: { order } })
      await load()
    } catch (e) {
      notify.fail(e, 'Failed to reorder')
    }
  }

  return {
    categories, rules, sanctions, servers, pending, scope, search,
    tree, loose, load, save, remove, move, serverName, sanctionsOf, notify,
  }
}
