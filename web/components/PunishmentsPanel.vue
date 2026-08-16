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

    <div class="grid gap-3">
      <div class="grid gap-3 sm:grid-cols-2">
        <label class="block">
          <span class="noro-label mb-1.5 block">Kind</span>
          <NoroSelect v-model="kind" class="w-full">
            <option value="warn">Warning</option>
            <option value="ban">Ban</option>
          </NoroSelect>
        </label>
        <label class="block">
          <span class="noro-label mb-1.5 block">Hours <span class="text-[var(--noro-muted)]">— empty = forever</span></span>
          <input v-model="hours" class="noro-input w-full" placeholder="forever" inputmode="numeric">
        </label>
      </div>

      <label class="block">
        <span class="noro-label mb-1.5 block">Reason</span>
        <textarea
          v-model="reason"
          rows="3"
          class="noro-input w-full resize-y"
          placeholder="What explains this punishment in six months"
        />
      </label>

      <div>
        <!-- Цвет повторяет тяжесть: перепутать бан с предупреждением в одно
             нажатие не должно быть легко. -->
        <AtomButton
          :variant="kind === 'ban' ? 'danger' : 'warning'"
          icon="i-lucide-gavel"
          :loading="busy"
          :disabled="reason.trim().length < 3"
          @click="create"
        >
          {{ kind === 'ban' ? 'Ban' : 'Warn' }}
        </AtomButton>
      </div>
    </div>

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
