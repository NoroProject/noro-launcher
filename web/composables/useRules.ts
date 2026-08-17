/**
 * Публичный свод правил: загрузка, переключение сервера и поиск.
 *
 * Свод грузится с сервера заново на каждое переключение, а не фильтруется на
 * месте: дополнения сервера в общий ответ не входят, и фильтрация показывала бы
 * пустоту там, где у сервера как раз есть свои пункты.
 */

import type { Rule, RulesResponse, RuleSanction, RuleScope } from '~/types/rules'

/** Пустой свод: страница остаётся рабочей, даже если мастер не ответил. */
const EMPTY: RulesResponse = { server_id: null, categories: [], rules: [], sanctions: [] }

export function useRules() {
  const api = useApi()
  const route = useRoute()
  const router = useRouter()
  const { locale } = useT()

  const serverId = ref<string>(String(route.query.server || ''))
  const search = ref<string>(String(route.query.q || ''))
  const scopes = ref<RuleScope[]>([])
  const failed = ref(false)

  const { data, pending, refresh } = useAsyncData(
    'public-rules',
    async () => {
      // Свод переводится на мастере: непереведённый пункт возвращается на
      // исходном языке, поэтому запрашивать его отдельно не нужно.
      const query = new URLSearchParams({ locale: locale.value })
      if (serverId.value) query.set('server_id', serverId.value)
      try {
        failed.value = false
        return await api.request<RulesResponse>(`/api/rules?${query}`)
      } catch {
        // Пустой свод вместо падения: страница объяснит, что правил нет,
        // и останется рабочей — она открыта и тем, у кого нет аккаунта.
        failed.value = true
        return EMPTY
      }
    },
    { default: () => EMPTY, watch: [serverId, locale] },
  )

  onMounted(async () => {
    try {
      scopes.value = await api.request<RuleScope[]>('/api/rules/scopes')
    } catch {
      scopes.value = []
    }
  })

  // Поиск и выбранный свод живут в адресе: ссылку на нужный пункт можно послать.
  watch([serverId, search], ([server, query]) => {
    router.replace({
      query: { ...(server ? { server } : {}), ...(query ? { q: query } : {}) },
    })
  })

  const categories = computed(() => data.value?.categories || [])
  const rules = computed<Rule[]>(() => data.value?.rules || [])
  const sanctions = computed<RuleSanction[]>(() => data.value?.sanctions || [])
  const tree = computed(() => buildRuleTree(categories.value, rules.value, search.value))
  const loose = computed(() => looseRules(rules.value, categories.value, search.value))
  const found = computed(
    () => tree.value.reduce((sum, node) => sum + node.total, 0) + loose.value.length,
  )

  return { serverId, search, scopes, tree, loose, found, rules, sanctions, pending, failed, refresh }
}
