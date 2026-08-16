<script setup lang="ts">
interface Punishment {
  id: string
  kind: string
  reason: string
  actor_label: string
  created_at: string
  expires_at: string | null
  revoked_at: string | null
}

const props = defineProps<{ userId: string }>()

const auth = useAuth()
const notify = useNotify()

const rows = ref<Punishment[]>([])
const kind = ref('warn')
const reason = ref('')
const hours = ref<string>('')
const busy = ref(false)

const load = async () => {
  rows.value = await auth.request<Punishment[]>(`/api/admin/users/${props.userId}/punishments`)
}

async function create() {
  busy.value = true
  try {
    await auth.request(`/api/admin/users/${props.userId}/punishments`, {
      method: 'POST',
      body: {
        kind: kind.value,
        reason: reason.value.trim(),
        hours: hours.value ? Number(hours.value) : null,
      },
    })
    reason.value = ''
    hours.value = ''
    notify.ok()
    await load()
  } catch (e) {
    notify.fail(e)
  } finally {
    busy.value = false
  }
}

async function revoke(id: string) {
  try {
    await auth.request(`/api/admin/users/${props.userId}/punishments/${id}`, { method: 'DELETE' })
    notify.ok()
    await load()
  } catch (e) {
    notify.fail(e)
  }
}

/** Снятое и истёкшее остаётся в списке: это тоже факт для следующего разбора. */
const active = (p: Punishment) =>
  !p.revoked_at && (!p.expires_at || new Date(p.expires_at) > new Date())

onMounted(() => load())
</script>

<template>
  <section class="noro-panel p-5 space-y-4">
    <h3 class="text-lg font-black text-[var(--noro-text)]">Punishments</h3>

    <div class="grid gap-2 sm:grid-cols-[120px_1fr_100px_auto]">
      <select v-model="kind" class="noro-input">
        <option value="warn">Warning</option>
        <option value="ban">Ban</option>
      </select>
      <input v-model="reason" class="noro-input" placeholder="Reason — it is what explains this in six months">
      <input v-model="hours" class="noro-input" placeholder="hours" inputmode="numeric">
      <AtomButton icon="i-lucide-gavel" :loading="busy" :disabled="reason.trim().length < 3" @click="create">
        Apply
      </AtomButton>
    </div>
    <p class="text-xs text-[var(--noro-muted)]">Leave hours empty for a permanent one.</p>

    <div v-if="rows.length" class="space-y-2">
      <div
        v-for="p in rows"
        :key="p.id"
        class="flex items-start justify-between gap-3 rounded border border-[var(--noro-border)] bg-[var(--noro-bg)] p-3 text-xs"
      >
        <div>
          <span
            class="rounded px-2 py-0.5 text-[10px] font-black uppercase tracking-wider"
            :class="active(p)
              ? 'bg-[color-mix(in_srgb,var(--noro-magenta)_16%,transparent)] text-[var(--noro-magenta)]'
              : 'bg-[var(--noro-input)] text-[var(--noro-muted)]'"
          >{{ p.kind }}{{ active(p) ? '' : p.revoked_at ? ' · revoked' : ' · expired' }}</span>
          <div class="mt-1 text-[var(--noro-text)]">{{ p.reason }}</div>
          <div class="text-[10px] text-[var(--noro-muted)]">
            {{ p.actor_label }} · {{ new Date(p.created_at).toLocaleString() }}
            <template v-if="p.expires_at"> · until {{ new Date(p.expires_at).toLocaleString() }}</template>
          </div>
        </div>
        <AtomButton
          v-if="active(p)"
          variant="dark"
          icon="i-lucide-undo-2"
          class="!min-h-8 !px-2"
          @click="revoke(p.id)"
        />
      </div>
    </div>
    <p v-else class="text-xs text-[var(--noro-muted)]">Nothing on record.</p>
  </section>
</template>
