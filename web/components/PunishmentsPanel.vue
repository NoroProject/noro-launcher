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
const { t } = useT()
const can = (perm: string) => auth.hasPermission(perm)
const canPunish = computed(() => auth.hasAny(
  'noro.mod.punish.warn', 'noro.mod.punish.mute',
  'noro.mod.punish.ban', 'noro.mod.punish.server_ban',
))
const notify = useNotify()

const rows = ref<Punishment[]>([])

const load = async () => {
  rows.value = await auth.request<Punishment[]>(`/api/admin/users/${props.userId}/punishments`)
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
    <h3 class="text-lg font-black text-[var(--noro-text)]">{{ t('admin-punish-title') }}</h3>

    <!-- Форма только тому, кто вообще может наказывать: остальным панель
         остаётся журналом. -->
    <PunishmentForm v-if="canPunish" :user-id="userId" @created="load" />

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
          >
            {{ kindLabel(p.kind, t) }}{{ active(p) ? '' : p.revoked_at ? ` · ${t('admin-punish-revoked')}` : ` · ${t('admin-punish-expired')}` }}
          </span>
          <div class="mt-1 text-[var(--noro-text)]">{{ p.reason }}</div>
          <div class="text-[10px] text-[var(--noro-muted)]">
            {{ p.actor_label }} · {{ new Date(p.created_at).toLocaleString() }}
            <template v-if="p.expires_at"> · {{ t('punishment-until', { date: new Date(p.expires_at).toLocaleString() }) }}</template>
          </div>
        </div>
        <AtomButton
          v-if="active(p) && can('noro.mod.punish.revoke')"
          variant="dark"
          icon="i-lucide-undo-2"
          class="!min-h-8 !px-2"
          @click="revoke(p.id)"
        />
      </div>
    </div>
    <p v-else class="text-xs text-[var(--noro-muted)]">{{ t('admin-punish-empty') }}</p>
  </section>
</template>
