<script setup lang="ts">
import type { LauncherStatus, SessionRow } from '~/types/sessions'

const props = defineProps<{ userId: string }>()
const emit = defineEmits<{ impersonate: []; requestLogs: [] }>()

const auth = useAuth()
const notify = useNotify()

const status = ref<LauncherStatus | null>(null)
const sessions = ref<SessionRow[]>([])
const pending = ref(false)

async function load() {
  pending.value = true
  try {
    status.value = await auth.request<LauncherStatus>(`/api/admin/users/${props.userId}/launcher`)
    sessions.value = await auth.request<SessionRow[]>(`/api/admin/users/${props.userId}/sessions`)
  } catch (e) {
    notify.fail(e, 'Failed to load launcher status')
  } finally {
    pending.value = false
  }
}

async function revokeAll() {
  try {
    const res = await auth.request<{ revoked: number }>(
      `/api/admin/users/${props.userId}/sessions`,
      { method: 'DELETE' }
    )
    notify.ok(`Revoked ${res.revoked} session(s)`)
    await load()
  } catch (e) {
    notify.fail(e)
  }
}

const seen = computed(() =>
  status.value?.last_seen_at ? new Date(status.value.last_seen_at).toLocaleString() : '—'
)

onMounted(() => load())
</script>

<template>
  <section class="noro-panel p-5 space-y-4">
    <div class="flex items-center justify-between">
      <h3 class="text-lg font-black text-[var(--noro-text)]">Launcher</h3>
      <AtomButton variant="dark" icon="i-lucide-refresh-cw" :loading="pending" class="!min-h-8 !px-2" @click="load" />
    </div>

    <div class="grid gap-2 text-xs">
      <div class="flex items-center gap-2">
        <span
          class="size-2 rounded-full"
          :class="status?.online ? 'bg-[var(--noro-blue)]' : 'bg-[var(--noro-muted)]'"
        />
        <span class="text-[var(--noro-text)]">{{ status?.online ? 'Online' : 'Offline' }}</span>
        <span class="text-[var(--noro-muted)]">· last seen {{ seen }}</span>
      </div>

      <div class="text-[var(--noro-muted)]">
        Version
        <span class="text-[var(--noro-text)]">{{ status?.version || '—' }}</span>
        <span
          v-if="status?.outdated"
          class="ml-2 rounded bg-[color-mix(in_srgb,var(--noro-cream)_16%,transparent)] px-1.5 py-0.5 text-[10px] font-black uppercase tracking-wider text-[var(--noro-cream)]"
        >outdated → {{ status?.current_version }}</span>
        <span v-if="status?.platform" class="ml-2">({{ status.platform }})</span>
      </div>

      <NuxtLink
        v-if="status?.open_integrity_flags"
        to="/admin/integrity"
        class="text-[var(--noro-magenta)] underline"
      >{{ status.open_integrity_flags }} unreviewed integrity flag(s)</NuxtLink>
    </div>

    <div class="flex flex-wrap gap-2">
      <AtomButton icon="i-lucide-user-check" @click="emit('impersonate')">Login as</AtomButton>
      <AtomButton variant="dark" icon="i-lucide-file-text" @click="emit('requestLogs')">
        Request logs
      </AtomButton>
      <AtomButton variant="dark" icon="i-lucide-log-out" @click="revokeAll">
        End all sessions
      </AtomButton>
    </div>

    <div v-if="sessions.length" class="space-y-1">
      <div class="noro-label">Active sessions</div>
      <div
        v-for="s in sessions"
        :key="s.id"
        class="flex items-center justify-between rounded border border-[var(--noro-border)] bg-[var(--noro-bg)] px-3 py-2 text-xs"
      >
        <span class="text-[var(--noro-muted)]">
          {{ s.scope }} · {{ new Date(s.created_at).toLocaleDateString() }}
          <span v-if="s.impersonated_by" class="text-[var(--noro-magenta)]">· impersonated</span>
        </span>
        <span class="text-[var(--noro-muted)]">until {{ new Date(s.expires_at).toLocaleDateString() }}</span>
      </div>
    </div>
  </section>
</template>
