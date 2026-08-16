<script setup lang="ts">
import type { AuditRow } from '~/types/audit'

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
      <AuditEntry v-for="row in rows" :key="row.id" :row="row" />

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
