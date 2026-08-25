<script setup lang="ts">
import type { LauncherStatus, SessionRow } from '~/types/sessions'

const props = defineProps<{ userId: string }>()
const emit = defineEmits<{ impersonate: []; requestLogs: [] }>()

const auth = useAuth()
const { t } = useT()
const can = (perm: string) => auth.hasPermission(perm)
const notify = useNotify()

const online = defineModel<boolean>('online', { default: false })
const status = ref<LauncherStatus | null>(null)
const sessions = ref<SessionRow[]>([])
const pending = ref(false)
const showSessions = ref(false)

async function load() {
  pending.value = true
  try {
    status.value = await auth.request<LauncherStatus>(`/api/admin/users/${props.userId}/launcher`)
    online.value = status.value.online
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
    <!-- Header with status badge & refresh -->
    <div class="flex items-center justify-between gap-3">
      <div class="flex items-center gap-2.5 min-w-0">
        <h3 class="text-base font-black tracking-tight text-[var(--noro-text)] shrink-0">
          {{ t('cabinet-launcher-title') }}
        </h3>
        <span
          class="inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-semibold transition-colors"
          :class="status?.online 
            ? 'bg-[color-mix(in_srgb,var(--noro-blue)_14%,transparent)] text-[var(--noro-blue)] border border-[color-mix(in_srgb,var(--noro-blue)_25%,transparent)]' 
            : 'bg-[var(--noro-input)] text-[var(--noro-muted)] border border-[var(--noro-border)]'"
        >
          <span
            class="size-1.5 rounded-full shrink-0"
            :class="status?.online ? 'bg-[var(--noro-blue)] animate-pulse' : 'bg-[var(--noro-muted)]'"
          />
          {{ status?.online ? t('web-rules-status-online') : t('web-rules-status-offline') }}
        </span>
      </div>

      <button
        type="button"
        class="inline-flex items-center justify-center rounded-lg border border-[var(--noro-border)] bg-[var(--noro-bg)] p-2 text-[var(--noro-muted)] transition hover:bg-[var(--noro-input)] hover:text-[var(--noro-text)] disabled:opacity-50"
        :disabled="pending"
        @click="load"
      >
        <UIcon name="i-lucide-refresh-cw" class="size-4" :class="{ 'animate-spin': pending }" />
      </button>
    </div>

    <!-- Metadata Cards Row -->
    <div class="grid grid-cols-2 gap-2 text-xs">
      <div class="rounded-xl border border-[var(--noro-border)] bg-[var(--noro-bg)] p-2.5 space-y-1">
        <div class="text-[10px] font-bold uppercase tracking-wider text-[var(--noro-muted)]">
          {{ t('admin-users-version') }}
        </div>
        <div class="flex items-center gap-1.5 flex-wrap font-mono font-medium text-[var(--noro-text)]">
          <span>{{ status?.version || '—' }}</span>
          <span v-if="status?.platform" class="text-[10px] text-[var(--noro-muted)]">({{ status.platform }})</span>
          <span
            v-if="status?.outdated"
            class="rounded bg-[color-mix(in_srgb,var(--noro-cream)_16%,transparent)] px-1.5 py-0.5 text-[9px] font-black uppercase text-[var(--noro-cream)]"
          >
            → {{ status?.current_version }}
          </span>
        </div>
      </div>

      <div class="rounded-xl border border-[var(--noro-border)] bg-[var(--noro-bg)] p-2.5 space-y-1">
        <div class="text-[10px] font-bold uppercase tracking-wider text-[var(--noro-muted)]">
          Last Activity
        </div>
        <div class="font-mono font-medium text-[var(--noro-text)] truncate" :title="seen">
          {{ seen }}
        </div>
      </div>
    </div>

    <!-- Integrity alert banner if flags exist -->
    <NuxtLink
      v-if="status?.open_integrity_flags"
      :to="adminLink.integrity()"
      class="flex items-center justify-between rounded-xl border border-[color-mix(in_srgb,var(--noro-magenta)_30%,transparent)] bg-[color-mix(in_srgb,var(--noro-magenta)_10%,transparent)] px-3 py-2 text-xs font-semibold text-[var(--noro-magenta)] transition hover:bg-[color-mix(in_srgb,var(--noro-magenta)_18%,transparent)]"
    >
      <span class="flex items-center gap-2">
        <UIcon name="i-lucide-shield-alert" class="size-4" />
        {{ status.open_integrity_flags }} unreviewed integrity flag(s)
      </span>
      <UIcon name="i-lucide-chevron-right" class="size-4" />
    </NuxtLink>

    <!-- Vertical Action Column -->
    <div class="flex flex-col gap-2 pt-1">
      <AtomButton
        v-if="can('noro.admin.users.impersonate')"
        variant="primary"
        icon="i-lucide-user-check"
        class="w-full !justify-start"
        @click="emit('impersonate')"
      >
        {{ t('admin-users-login-as') }}
      </AtomButton>

      <AtomButton
        v-if="can('noro.admin.support.request')"
        variant="dark"
        icon="i-lucide-file-text"
        class="w-full !justify-start"
        @click="emit('requestLogs')"
      >
        {{ t('admin-users-req-logs') }}
      </AtomButton>

      <AtomButton
        v-if="can('noro.admin.users.sessions.revoke')"
        variant="dark"
        icon="i-lucide-log-out"
        class="w-full !justify-start"
        @click="revokeAll"
      >
        {{ t('admin-users-end-sessions') }}
      </AtomButton>
    </div>

    <!-- Active Sessions Accordion -->
    <div v-if="sessions.length" class="border-t border-[var(--noro-border)] pt-3">
      <button
        type="button"
        class="flex w-full items-center justify-between text-xs text-[var(--noro-muted)] transition hover:text-[var(--noro-text)]"
        @click="showSessions = !showSessions"
      >
        <span class="flex items-center gap-2 font-semibold">
          <UIcon name="i-lucide-key-round" class="size-3.5" />
          {{ sessions.length }} active session{{ sessions.length === 1 ? '' : 's' }}
        </span>
        <UIcon :name="showSessions ? 'i-lucide-chevron-up' : 'i-lucide-chevron-down'" class="size-3.5" />
      </button>

      <div v-if="showSessions" class="mt-2.5 noro-scroll max-h-52 space-y-1.5 overflow-y-auto pr-1">
        <div
          v-for="s in sessions"
          :key="s.id"
          class="flex items-center justify-between rounded-lg border border-[var(--noro-border)] bg-[var(--noro-bg)] px-3 py-2 text-xs"
        >
          <div class="flex items-center gap-2 min-w-0">
            <span class="font-medium text-[var(--noro-text)] truncate">{{ s.scope }}</span>
            <span v-if="s.impersonated_by" class="rounded bg-[color-mix(in_srgb,var(--noro-magenta)_20%,transparent)] px-1.5 py-0.5 text-[9px] font-bold text-[var(--noro-magenta)]">
              impersonated
            </span>
          </div>
          <span class="text-[10px] text-[var(--noro-muted)] shrink-0 font-mono">
            until {{ new Date(s.expires_at).toLocaleDateString() }}
          </span>
        </div>
      </div>
    </div>
  </section>
</template>
