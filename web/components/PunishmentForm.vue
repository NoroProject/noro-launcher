<script setup lang="ts">
/**
 * Выдача наказания. Порядок шагов тот же, что в голове у модератора: за какое
 * правило, что именно за него полагается, кому и почему.
 */
import type { ServerRow } from '~/types/api'

const props = defineProps<{ userId: string, caseId?: string }>()
const emit = defineEmits<{ created: [] }>()

const auth = useAuth()
const notify = useNotify()
const { t } = useT()

const serverId = ref('')
const reason = ref('')
const busy = ref(false)
const servers = ref<ServerRow[]>([])

const form = usePunishForm(serverId)
const { ruleId, rule, options, sanction, kind, duration, minutes, problem, canBypass } = form

const scoped = computed(() => kind.value === 'server_ban' || kind.value === 'mute')
const blocked = computed(() => !!problem.value || reason.value.trim().length < 3
  || (kind.value === 'server_ban' && !serverId.value))

onMounted(async () => {
  try {
    servers.value = await auth.requestList<ServerRow>('/api/admin/servers')
  } catch {
    // Список серверов нужен только для серверного бана: без него форма
    // остаётся рабочей для остальных видов.
    servers.value = []
  }
})

/** Причина набирается сама из правила — её всё ещё можно дописать руками. */
watch(rule, (picked) => {
  if (picked) reason.value = `[${picked.code}] ${picked.title}`
})

async function create() {
  busy.value = true
  try {
    await auth.request(`/api/admin/users/${props.userId}/punishments`, {
      method: 'POST',
      body: {
        kind: kind.value,
        reason: reason.value.trim(),
        minutes: minutes.value,
        server_id: scoped.value && serverId.value ? serverId.value : null,
        rule_id: ruleId.value || null,
        // Наказание из разбора встаёт в его ленту.
        case_id: props.caseId || null,
      },
    })
    reason.value = ''
    duration.value = ''
    ruleId.value = ''
    notify.ok()
    emit('created')
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="grid min-w-0 grid-cols-[minmax(0,1fr)] gap-4">
    <PunishmentRulePicker v-model="ruleId" :rules="form.rules.value" :server-id="serverId" />

    <!-- Варианты правила: вид и вилка срока. Всё, что вне их, требует байпаса. -->
    <div v-if="rule" class="grid gap-2">
      <span class="noro-label">{{ t('admin-punish-allowed-for-rule') }}</span>
      <div v-if="options.length" class="flex flex-wrap gap-2">
        <button
          v-for="option in options"
          :key="option.id"
          type="button"
          class="noro-chip px-3 py-2 text-xs font-bold"
          :class="sanction?.id === option.id ? 'noro-chip-on' : 'text-[var(--noro-muted)]'"
          @click="form.pick(option)"
        >
          {{ formatSanction(option, t) }}
          <span v-if="option.label" class="ml-1 opacity-60">· {{ option.label }}</span>
        </button>
      </div>
      <p v-else class="text-sm font-medium text-[var(--noro-muted)]">
        {{ t('admin-punish-none-allowed') }}
      </p>
    </div>

    <!-- Одна колонка, а не две: поле срока висит на условии, и в сетке из
         двух колонок выбор «Предупреждение» оставлял «Вид» половиной пустой
         строки — ширина прыгала на каждом переключении. -->
    <div class="grid gap-4">
      <label class="block min-w-0">
        <span class="noro-label mb-2 block">{{ t('admin-punish-kind') }}</span>
        <NoroSelect v-model="kind" class="w-full" :disabled="!canBypass && !!sanction">
          <option v-for="option in form.allowedKinds.value" :key="option" :value="option">
            {{ kindLabel(option, t) }}
          </option>
        </NoroSelect>
      </label>

      <PunishmentDuration v-if="kind !== 'warn'" v-model="duration" />
    </div>

    <label v-if="scoped" class="block">
      <span class="noro-label mb-2 block">
        {{ t('admin-punish-target-server') }} {{ kind === 'mute' ? t('admin-punish-target-server-optional') : '' }}
      </span>
      <NoroSelect v-model="serverId" class="w-full">
        <option value="">{{ kind === 'mute' ? t('admin-punish-target-all-servers') : t('admin-punish-target-select-server') }}</option>
        <option v-for="s in servers" :key="s.id" :value="s.id">{{ s.name }}</option>
      </NoroSelect>
    </label>

    <label class="block min-w-0">
      <span class="noro-label mb-2 block">{{ t('admin-punish-reason') }}</span>
      <textarea
        v-model="reason"
        rows="3"
        class="noro-input w-full resize-y"
        :placeholder="t('admin-punish-reason-placeholder')"
      />
    </label>

    <!-- Почему кнопка не нажимается — на месте, а не после отказа сервера. -->
    <p v-if="problem" class="text-sm font-bold text-[var(--noro-magenta)]">{{ problem }}</p>

    <div class="grid gap-2">
      <AtomButton
        class="w-full"
        :variant="kind === 'ban' || kind === 'server_ban' ? 'danger' : 'warning'"
        icon="i-lucide-gavel"
        :loading="busy"
        :disabled="blocked"
        @click="create"
      >
        {{ kindLabel(kind, t) }}
      </AtomButton>
      <span v-if="canBypass" class="noro-label">{{ t('admin-punish-bypass-hint') }}</span>
    </div>
  </div>
</template>
