<script setup lang="ts">
import type { AuditRow } from '~/types/audit'

interface ActionInfo {
  name: string
  group: string
  title: string
}

const auth = useAuth()
const notify = useNotify()
await auth.loadMe()

const action = ref('')
const targetKind = ref('')
const targetId = ref('')
const rows = ref<AuditRow[]>([])
const pending = ref(false)
const done = ref(false)

const actions = ref<ActionInfo[]>([])
const groups = ref<string[]>([])
const targetKinds = ref<string[]>([])

const PAGE = 50

/** Действия, сгруппированные для `<optgroup>`: плоский список из сорока пунктов не читается. */
const grouped = computed(() =>
  groups.value
    .map((g) => ({ group: g, items: actions.value.filter((a) => a.group === g) }))
    .filter((g) => g.items.length)
    .concat(
      // Всё, чья группа не объявлена, — в конец, а не в никуда.
      actions.value.some((a) => !groups.value.includes(a.group))
        ? [{ group: 'Other', items: actions.value.filter((a) => !groups.value.includes(a.group)) }]
        : []
    )
)

function query(beforeId?: number) {
  const p = new URLSearchParams({ limit: String(PAGE) })
  if (action.value) p.set('action', action.value)
  if (targetKind.value) p.set('target_kind', targetKind.value)
  if (targetId.value.trim()) p.set('target_id', targetId.value.trim())
  if (beforeId) p.set('before_id', String(beforeId))
  return `/api/admin/audit?${p}`
}

async function load(more = false) {
  pending.value = true
  try {
    const cursor = more ? rows.value[rows.value.length - 1]?.id : undefined
    const page = await auth.request<AuditRow[]>(query(cursor))
    rows.value = more ? [...rows.value, ...page] : page
    // Неполная страница означает конец: следующий запрос вернёт пустоту.
    done.value = page.length < PAGE
  } catch (e) {
    notify.fail(e, 'Failed to load the audit log')
  } finally {
    pending.value = false
  }
}

async function loadActions() {
  try {
    const res = await auth.request<{
      actions: ActionInfo[]
      groups: string[]
      target_kinds: string[]
    }>('/api/admin/audit/actions')
    actions.value = res.actions
    groups.value = res.groups
    targetKinds.value = res.target_kinds
  } catch {
    // Фильтр без справочника всё ещё работает по «всем событиям».
  }
}

function reset() {
  action.value = ''
  targetKind.value = ''
  targetId.value = ''
  load()
}

/** Человеческое имя события — по нему и подписаны строки журнала. */
const titles = computed(() => Object.fromEntries(actions.value.map((a) => [a.name, a.title])))

watch([action, targetKind], () => load())
onMounted(() => {
  loadActions()
  load()
})
</script>

<template>
  <NoroShell title="AUDIT" subtitle="Who changed what, and when">
    <template #actions>
      <AtomButton icon="i-lucide-refresh-cw" variant="dark" :loading="pending" @click="load()">
        Refresh
      </AtomButton>
    </template>

    <section class="noro-panel mb-4 grid gap-4 p-4 md:grid-cols-[1fr_200px_1fr_auto]">
      <label class="block">
        <span class="noro-label mb-1.5 block">Event</span>
        <NoroSelect v-model="action" class="w-full">
          <option value="">All events</option>
          <optgroup v-for="g in grouped" :key="g.group" :label="g.group">
            <option v-for="a in g.items" :key="a.name" :value="a.name">{{ a.title }}</option>
          </optgroup>
        </NoroSelect>
      </label>

      <label class="block">
        <span class="noro-label mb-1.5 block">Target</span>
        <NoroSelect v-model="targetKind" class="w-full">
          <option value="">Anything</option>
          <option v-for="k in targetKinds" :key="k" :value="k">{{ k }}</option>
        </NoroSelect>
      </label>

      <label class="block">
        <span class="noro-label mb-1.5 block">Target id <span class="text-[var(--noro-muted)]">— optional</span></span>
        <input
          v-model="targetId"
          class="noro-input w-full"
          placeholder="UUID"
          @keyup.enter="load()"
        >
      </label>

      <div class="flex items-end gap-2">
        <AtomButton icon="i-lucide-search" :loading="pending" @click="load()">Apply</AtomButton>
        <AtomButton variant="dark" icon="i-lucide-x" @click="reset">Reset</AtomButton>
      </div>
    </section>

    <div v-if="rows.length" class="grid gap-2">
      <AuditEntry v-for="row in rows" :key="row.id" :row="row" :title="titles[row.action]" />

      <AtomButton
        v-if="!done"
        variant="dark"
        icon="i-lucide-chevron-down"
        :loading="pending"
        class="mt-2"
        @click="load(true)"
      >
        Load more
      </AtomButton>
    </div>

    <EmptyState
      v-else-if="!pending"
      icon="i-lucide-scroll-text"
      title="Nothing recorded yet"
      text="Logins, launches, integrity findings and every admin action land here as they happen."
    />
  </NoroShell>
</template>
