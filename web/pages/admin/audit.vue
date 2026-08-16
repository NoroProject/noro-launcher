<script setup lang="ts">
interface AuditRow {
  id: number
  at: string
  actor_id: string | null
  actor_label: string
  action: string
  target_kind: string | null
  target_id: string | null
  details: Record<string, unknown>
  ip: string | null
}

const auth = useAuth()
const notify = useNotify()
await auth.loadMe()

const action = ref('')
const targetId = ref('')
const rows = ref<AuditRow[]>([])
const pending = ref(false)
const done = ref(false)

const PAGE = 50

function query(beforeId?: number) {
  const p = new URLSearchParams({ limit: String(PAGE) })
  if (action.value.trim()) p.set('action', action.value.trim())
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

/** Группа события — по ней красится метка: user.ban → user. */
const groupOf = (a: string) => a.split('.')[0]

const groupColor: Record<string, string> = {
  user: 'var(--noro-magenta)',
  role: 'var(--noro-blue)',
  build: 'var(--noro-cream)',
  server: 'var(--noro-cream)',
  launcher: 'var(--noro-blue)',
  admin_token: 'var(--noro-magenta)',
  storage: 'var(--noro-muted)',
}

const hasDetails = (d: Record<string, unknown>) =>
  Object.values(d ?? {}).some((v) => v !== null && v !== undefined && v !== '')

onMounted(() => load())
</script>

<template>
  <NoroShell title="AUDIT" subtitle="Who changed what, and when">
    <template #actions>
      <AtomButton icon="i-lucide-refresh-cw" variant="dark" :loading="pending" @click="load()">
        Refresh
      </AtomButton>
    </template>

    <section class="noro-panel mb-4 grid gap-4 p-4 md:grid-cols-[1fr_1fr_auto]">
      <label class="block">
        <span class="noro-label mb-1.5 block">Action prefix</span>
        <input
          v-model="action"
          class="noro-input w-full"
          placeholder="user, user.ban, build.publish"
          @keyup.enter="load()"
        >
      </label>
      <label class="block">
        <span class="noro-label mb-1.5 block">Target id</span>
        <input
          v-model="targetId"
          class="noro-input w-full"
          placeholder="UUID of a user, build or server"
          @keyup.enter="load()"
        >
      </label>
      <div class="flex items-end">
        <AtomButton icon="i-lucide-search" :loading="pending" @click="load()">Apply</AtomButton>
      </div>
    </section>

    <div v-if="rows.length" class="grid gap-2">
      <article
        v-for="row in rows"
        :key="row.id"
        class="rounded-[var(--noro-r-sm)] border border-[var(--noro-border)] bg-[var(--noro-bg)] p-4"
      >
        <div class="flex flex-wrap items-center gap-3">
          <span
            class="rounded px-2 py-1 text-[10px] font-black uppercase tracking-wider"
            :style="{
              color: groupColor[groupOf(row.action)] || 'var(--noro-muted)',
              background: `color-mix(in srgb, ${groupColor[groupOf(row.action)] || 'var(--noro-muted)'} 16%, transparent)`,
            }"
          >{{ row.action }}</span>

          <span class="text-sm font-bold text-[var(--noro-text)]">{{ row.actor_label }}</span>

          <span v-if="row.target_kind" class="text-xs text-[var(--noro-muted)]">
            → {{ row.target_kind }}
            <NuxtLink
              v-if="row.target_kind === 'user'"
              :to="`/admin/users/${row.target_id}`"
              class="underline hover:text-[var(--noro-cream)]"
            >{{ row.target_id }}</NuxtLink>
            <template v-else>{{ row.target_id }}</template>
          </span>

          <time class="ml-auto text-xs text-[var(--noro-muted)]">
            {{ new Date(row.at).toLocaleString() }}
          </time>
        </div>

        <pre
          v-if="hasDetails(row.details)"
          class="noro-scroll mt-3 overflow-x-auto rounded bg-[var(--noro-input)] p-3 text-xs text-[var(--noro-muted)]"
        >{{ JSON.stringify(row.details, null, 2) }}</pre>
      </article>

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
      text="Admin actions land here as they happen — role grants, bans, deployments."
    />
  </NoroShell>
</template>
