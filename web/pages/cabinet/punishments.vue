<script setup lang="ts">
const auth = useAuth()
const { t } = useT()
await auth.loadMe()

const punishments = ref<any[]>([])
const loading = ref(false)

async function loadPunishments() {
  loading.value = true
  try {
    punishments.value = await auth.request<any[]>('/api/me/punishments')
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
}

function isPunishmentActive(p: any) {
  if (p.revoked_at) return false
  if (!p.expires_at) return true
  return new Date(p.expires_at).getTime() > Date.now()
}

function getKindLabel(kind: string) {
  switch (kind) {
    case 'ban': return t('punishment-kind-ban')
    case 'warn': return t('punishment-kind-warn')
    case 'mute': return t('punishment-kind-mute')
    case 'server_ban': return t('punishment-kind-server-ban')
    default: return kind.toUpperCase()
  }
}

onMounted(() => loadPunishments())
</script>

<template>
  <NoroShell title="PUNISHMENTS" subtitle="History of warnings, bans, and restrictions">
    <div class="space-y-4">
      <section class="noro-panel p-6 space-y-4">
        <div class="flex items-center justify-between">
          <div>
            <h2 class="text-lg font-bold text-[var(--noro-text)] flex items-center gap-2">
              <UIcon name="i-lucide-alert-triangle" class="size-5 text-amber-400" />
              {{ t('cabinet-punishments-title') }}
            </h2>
            <p class="text-xs text-[var(--noro-muted)]">
              {{ t('cabinet-punishments-lead') }}
            </p>
          </div>
          <AtomButton variant="secondary" icon="i-lucide-refresh-cw" :loading="loading" @click="loadPunishments">
            {{ t('cabinet-punishments-refresh') }}
          </AtomButton>
        </div>

        <div v-if="punishments.length" class="grid gap-3">
          <div
            v-for="p in punishments"
            :key="p.id"
            class="flex flex-wrap items-center justify-between gap-4 rounded-xl border border-[var(--noro-border)] bg-[var(--noro-bg)] p-4 text-xs transition hover:border-[var(--noro-cream)]/30"
          >
            <div class="flex items-center gap-3 min-w-0">
              <span
                class="px-2.5 py-1 rounded-md text-[11px] font-bold uppercase tracking-wider shrink-0"
                :class="{
                  'bg-red-500/20 text-red-400 border border-red-500/30': p.kind === 'ban',
                  'bg-amber-500/20 text-amber-300 border border-amber-500/30': p.kind === 'warn',
                  'bg-cyan-500/20 text-cyan-300 border border-cyan-500/30': p.kind === 'mute',
                  'bg-purple-500/20 text-purple-300 border border-purple-500/30': p.kind === 'server_ban'
                }"
              >
                {{ getKindLabel(p.kind) }}
              </span>
              <div class="min-w-0 space-y-0.5">
                <div class="font-bold text-[var(--noro-text)] text-sm truncate">{{ p.reason }}</div>
                <div class="text-xs text-[var(--noro-muted)]">
                  {{ t('punishment-actor', { actor: p.actor_label }) }}
                  · {{ new Date(p.created_at).toLocaleString() }}
                  <span v-if="p.expires_at"> · {{ t('punishment-until', { date: new Date(p.expires_at).toLocaleString() }) }}</span>
                  <span v-else> · {{ t('punishment-forever') }}</span>
                </div>
              </div>
            </div>

            <div class="shrink-0">
              <span
                v-if="p.revoked_at"
                class="rounded-md bg-gray-500/20 border border-gray-500/30 px-3 py-1 text-xs font-semibold text-gray-400"
              >
                {{ t('punishment-status-revoked') }}
              </span>
              <span
                v-else-if="isPunishmentActive(p)"
                class="rounded-md bg-red-500/20 border border-red-500/30 px-3 py-1 text-xs font-bold text-red-400"
              >
                {{ t('punishment-status-active') }}
              </span>
              <span
                v-else
                class="rounded-md bg-gray-500/20 border border-gray-500/30 px-3 py-1 text-xs font-semibold text-gray-400"
              >
                {{ t('punishment-status-expired') }}
              </span>
            </div>
          </div>
        </div>

        <EmptyState
          v-else
          icon="i-lucide-check-circle-2"
          :title="t('cabinet-punishments-none-title')"
          :text="t('cabinet-punishments-none-text')"
        />
      </section>
    </div>
  </NoroShell>
</template>
