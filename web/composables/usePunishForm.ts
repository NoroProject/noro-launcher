/**
 * Форма наказания: правило задаёт рамки, модератор выбирает внутри них.
 *
 * Те же проверки живут на мастере — здесь они нужны, чтобы кнопка гасла до
 * отправки, а не после отказа с сервера.
 */

import type { PunishmentKind, Rule, RuleSanction, RulesResponse } from '~/types/rules'

export function usePunishForm(serverId: Ref<string>) {
  const api = useApi()
  const auth = useAuth()
  const { t } = useT()

  const rules = ref<Rule[]>([])
  const sanctions = ref<RuleSanction[]>([])
  const ruleId = ref('')
  const sanctionId = ref('')
  const kind = ref<PunishmentKind>('warn')
  const duration = ref('')

  /** Байпас снимает рамки свода: старший модератор решает по обстоятельствам. */
  const canBypass = computed(() => auth.hasPermission('noro.mod.punish.bypass'))
  const canPermanent = computed(() => auth.hasPermission('noro.mod.punish.permanent') || canBypass.value)
  const allowedKinds = computed(() =>
    (['warn', 'mute', 'ban', 'server_ban'] as const).filter(k => auth.hasPermission(`noro.mod.punish.${k}`)),
  )

  // Свод перезапрашивается при смене сервера: у сервера бывают свои пункты.
  watch(serverId, async (server) => {
    try {
      const query = server ? `?server_id=${server}` : ''
      const data = await api.request<RulesResponse>(`/api/rules${query}`)
      rules.value = data.rules
      sanctions.value = data.sanctions
    } catch {
      rules.value = []
      sanctions.value = []
    }
  }, { immediate: true })

  const rule = computed(() => rules.value.find(r => r.id === ruleId.value) || null)
  /** Варианты правила, доступные этому модератору по виду наказания. */
  const options = computed(() =>
    sanctions.value
      .filter(s => s.rule_id === ruleId.value)
      .filter(s => allowedKinds.value.includes(s.kind)),
  )
  const sanction = computed(() => options.value.find(s => s.id === sanctionId.value) || null)

  const minutes = computed(() => parseDuration(duration.value))
  const broken = computed(() => Number.isNaN(minutes.value))

  /** Почему нельзя отправить — одной строкой, до нажатия кнопки. */
  const problem = computed(() => {
    if (broken.value) return 'The term is not understood: try 7d, 12h, 1d 6h, 30m.'
    if (!allowedKinds.value.includes(kind.value)) return 'You cannot issue this kind of punishment.'
    if (minutes.value === null && kind.value !== 'warn' && !canPermanent.value) {
      return 'Punishing forever needs noro.mod.punish.permanent.'
    }
    if (canBypass.value) return ''
    if (!rule.value) return 'Pick a rule: without it only a bypass permission can punish.'
    if (!options.value.length) return 'This rule sets no punishment you are allowed to issue.'
    if (!sanction.value) return 'Pick one of the punishments the rule allows.'
    if (sanction.value.kind !== kind.value) return 'The chosen option is for another kind of punishment.'
    if (!sanctionAllows(sanction.value, minutes.value)) {
      return `The rule allows ${formatSanction(sanction.value, t)}.`
    }
    return ''
  })

  /** Выбор варианта заполняет вид и срок: чаще всего его и оставляют. */
  function pick(option: RuleSanction) {
    sanctionId.value = option.id
    kind.value = option.kind
    const preset = defaultMinutes(option)
    duration.value = preset ? formatDuration(preset) : ''
  }

  watch(ruleId, () => {
    sanctionId.value = ''
    if (options.value.length === 1) pick(options.value[0]!)
  })

  return {
    rules, ruleId, rule, options, sanction, sanctionId, kind, duration,
    minutes, broken, problem, canBypass, allowedKinds, pick,
  }
}
